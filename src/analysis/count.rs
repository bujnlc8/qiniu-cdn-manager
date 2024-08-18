//! 查询请求次数

#![allow(clippy::too_many_arguments)]

use std::{cmp::Ordering, path::PathBuf};

use anyhow::anyhow;
use chrono::Local;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

use super::Freq;
use crate::{
    utils::{print_err, qy_robot::QyRobot},
    Client, NOT_FOUND_MSG,
};
use colored::Colorize;

#[derive(Debug, Deserialize)]
pub struct ReqCountResponse {
    pub code: i32,
    pub error: String,
    pub data: Option<ReqCountData>,
}

#[derive(Debug, Deserialize)]
pub struct ReqCountData {
    pub points: Vec<String>,
    #[serde(rename = "reqCount")]
    pub req_count: Vec<i64>,
}

#[derive(Debug, Serialize)]
pub struct ReqCountParam<'a> {
    pub domains: Vec<String>,
    pub freq: &'a str,
    pub region: &'a str,
    #[serde(rename = "startDate")]
    pub start_date: &'a str,
    #[serde(rename = "endDate")]
    pub end_date: &'a str,
}

impl Client {
    /// ### [查询请求次数](https://developer.qiniu.com/fusion/4081/cdn-log-analysis#8)
    pub async fn req_count(
        &self,
        freq: Freq,
        region: &str,
        start_date: &str,
        end_date: &str,
        domains: Vec<String>,
    ) -> Result<ReqCountResponse, anyhow::Error> {
        let url = format!("https://{}{}", self.host, "/v2/tune/loganalyze/reqcount");
        let freq = freq.to_string();
        let data = ReqCountParam {
            domains,
            freq: freq.as_str(),
            region,
            start_date,
            end_date,
        };
        let response = self
            .do_request::<ReqCountResponse, ReqCountParam>(
                "POST",
                &url,
                None,
                Some("application/json"),
                Some(&data),
            )
            .await?;
        Ok(response)
    }

    pub async fn print_count(
        &self,
        response: ReqCountResponse,
        limit: Option<i32>,
        regions: &str,
        start_date: &str,
        end_date: &str,
        freq: Freq,
        no_warn: bool,
        domains: Vec<String>,
    ) -> Result<i64, anyhow::Error> {
        let limit = limit.unwrap_or(10000);
        match limit.cmp(&0) {
            Ordering::Less => return Err(anyhow!("limit参数错误")),
            Ordering::Equal => {
                println!(
                    "域名 {} 区域 {} {}～{} 请求次数: ",
                    domains.join(",").yellow().bold(),
                    regions.bold().yellow(),
                    start_date.bold(),
                    end_date.bold(),
                );
            }
            Ordering::Greater => {
                println!(
                    "域名 {} 区域 {} {}～{} 请求次数如下: ",
                    domains.join(",").yellow().bold(),
                    regions.bold().yellow(),
                    start_date.bold(),
                    end_date.bold(),
                );
                println!(
                    "{:^20} {:^10}",
                    "Time".bold().green(),
                    "Count".bold().green(),
                );
            }
        }
        if response.code != 200 {
            let msg = format!("code: {}, message: {}", response.code, response.error);
            print_err(msg.as_str(), false);
            return Ok(0);
        }
        if response.data.is_none() {
            print_err(NOT_FOUND_MSG, false);
            return Ok(0);
        }
        let data = response.data.unwrap();
        let now = Local::now().format("%Y-%m-%d-%H-%M").to_string();
        let mut total = 0;
        let mut row_counts = vec![];
        for (i, t) in data.points.iter().enumerate() {
            let c = *data.req_count.get(i).unwrap();
            if &now < t && c == 0 {
                continue;
            }
            if (i + 1) as i32 <= limit {
                println!("{:^20} {:^10}", t, c,);
                total += c;
            }
            if limit == 0 {
                total += c;
            }
            if c > 0 {
                row_counts.push((c, t.to_owned()));
            }
        }
        if total >= 0 {
            println!("{:^20} {:^10}", "Total".bold().green(), total,);
        }
        // 如果是5分钟的粒度，最后一条有次数的数据量超过次数配置，通过企业微信告警
        if freq == Freq::FiveMin
            && !row_counts.is_empty()
            && regions == "global"
            && !no_warn
            && self.config.monitor.qy_robot.is_some()
        {
            let warn_count = self.config.five_minute_count.unwrap_or(1000);
            let mut skip = 0;
            if row_counts.len() > 5 {
                skip = row_counts.len() - 5;
            }
            for (count, t) in row_counts.iter().skip(skip) {
                if count.ge(&warn_count) {
                    let send_mark_file_path = PathBuf::from(
                        format!(
                            "/tmp/qiniu/monitor/count/{}{}{}",
                            domains.join(","),
                            t,
                            count
                        )
                        .replace(" ", ""),
                    );
                    if send_mark_file_path.exists() {
                        continue;
                    }
                    let msg = format!(
                        "## 🚨七牛CDN流量告警\n\n域名`{}`在`{}` 5分钟内的请求次数为`{}`次, 超过告警值`{}`次，请留意！",
                        domains.join(","),
                    t,
                    count,
                        warn_count,
                    );
                    if QyRobot::new(self.config.monitor.qy_robot.clone().unwrap())
                        .send_message(&msg)
                        .await
                        .is_err()
                    {
                        print_err(format!("消息发送失败: {}", msg).as_str(), false);
                    } else {
                        let send_mark_dir = PathBuf::from("/tmp/qiniu/monitor/count");
                        if !send_mark_dir.exists() {
                            fs::create_dir_all(send_mark_dir).await?;
                        }
                        File::create(send_mark_file_path).await?.write_i8(1).await?;
                    }
                }
            }
        }
        Ok(row_counts.iter().map(|x| x.0).sum())
    }

    pub async fn all_domain_req_count(
        &mut self,
        freq: Freq,
        region: &str,
        start_date: &str,
        end_date: &str,
        no_warn: bool,
        domains: Vec<String>,
    ) -> Result<(), anyhow::Error> {
        let mut total = 0;
        for d in domains {
            if self.config.debug.unwrap_or(false) {
                println!("[DEBUG] monitor domain: {}", d)
            }
            let response = self
                .req_count(freq, region, start_date, end_date, vec![d.clone()])
                .await?;
            match self
                .print_count(
                    response,
                    Some(0),
                    region,
                    start_date,
                    end_date,
                    freq,
                    no_warn,
                    vec![d.clone()],
                )
                .await
            {
                Ok(t) => total += t,
                Err(e) => {
                    print_err(e.to_string().as_str(), false);
                }
            }
        }
        println!("\n{}{}", "所有域名请求次数总计: ".red().bold(), total);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::config::Config;

    use super::*;

    #[tokio::test]
    async fn req_count_test() {
        let config = Config::parse(Some(PathBuf::from("./qiniu-cdn.toml")));
        let analysis = Client::new(&config, crate::SubFunctionEnum::AnalysisCount);
        let response = analysis
            .req_count(
                Freq::OneDay,
                "china",
                "2024-07-16",
                "2024-07-16",
                vec![config.cdn.domain.clone()],
            )
            .await
            .unwrap();
        println!("{:#?}", response)
    }
}
