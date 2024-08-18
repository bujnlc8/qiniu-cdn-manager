//! 流量带宽

#![allow(clippy::too_many_arguments)]

use std::{collections::HashMap, ops::Div, path::PathBuf};

use chrono::Local;
use serde::Deserialize;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

use crate::{
    utils::{print_err, qy_robot::QyRobot},
    Client, NOT_FOUND_MSG,
};
use colored::Colorize;

/// 计费流量响应
#[derive(Debug, Deserialize)]
pub struct ChargeTrafficResponse {
    pub code: Option<i32>,
    pub error: String,
    pub time: Option<Vec<String>>,
    pub data: Option<HashMap<String, DomainTraffic>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DomainTraffic {
    pub china: Option<Vec<i64>>,
    pub oversea: Option<Vec<i64>>,
}

impl Client {
    /// [查询 cdn 计费流量](https://developer.qiniu.com/fusion/1230/traffic-bandwidth#4)
    pub async fn charge_traffic(
        &self,
        start_date: &str,
        end_date: &str,
        granularity: &str,
        domain: &str,
    ) -> Result<ChargeTrafficResponse, anyhow::Error> {
        let url = format!("https://{}{}", self.host, "/v2/tune/flux");
        let mut data = HashMap::new();
        data.insert("startDate", start_date);
        data.insert("endDate", end_date);
        data.insert("granularity", granularity);
        data.insert("domains", domain);
        let response = self
            .do_request::<ChargeTrafficResponse, HashMap<&str, &str>>(
                "POST",
                &url,
                None,
                Some("application/json"),
                Some(&data),
            )
            .await?;
        Ok(response)
    }

    pub async fn print_traffic_result(
        &self,
        response: &ChargeTrafficResponse,
        no_print: bool,
        start_date: &str,
        end_date: &str,
        granularity: &str,
        no_warn: bool,
        domain: &str,
    ) -> Result<f64, anyhow::Error> {
        if response.code.is_none() || response.code.unwrap() != 200 {
            eprintln!(
                "[ERR] code: {:?}, message: {}.",
                response.code,
                response.error.red()
            );
            return Ok(0.0);
        }
        if response.time.clone().is_none()
            || response.time.clone().unwrap().is_empty()
            || response.data.is_none()
        {
            print_err(NOT_FOUND_MSG, false);
            return Ok(0.0);
        }
        let binding = response.data.clone().unwrap();
        let data = binding.get(domain);
        if data.is_none() {
            print_err(NOT_FOUND_MSG, false);
            return Ok(0.0);
        }
        if !no_print {
            println!(
                "域名 {} {}～{} 流量如下: ",
                domain.bold().yellow(),
                start_date.bold(),
                end_date.bold(),
            );
            println!(
                "{:^20} {:^20} {:^20}",
                "Time".bold().green(),
                "China(MB)".bold().green(),
                "Oversea(MB)".bold().green(),
            );
        }
        let data = data.unwrap();
        let china = data.china.clone().unwrap_or_default();
        let oversea = data.oversea.clone().unwrap_or_default();
        let mut china_total = 0f64;
        let mut oversea_total = 0f64;
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let mut traffic_rows = vec![];
        for (i, t) in response.time.clone().unwrap().iter().enumerate() {
            let china_traffic = match china.get(i) {
                Some(v) => (*v as f64).div(1024.0).div(1024.0),
                None => 0f64,
            };
            let oversea_traffic = match oversea.get(i) {
                Some(v) => (*v as f64).div(1024.0).div(1024.0),
                None => 0f64,
            };
            if t > &now && china_traffic == 0.0 && oversea_traffic == 0.0 {
                continue;
            }
            if !no_print {
                let china_percent = china_traffic / (china_traffic + oversea_traffic) * 100.0;
                println!(
                    "{:^20} {:^20} {:^20}",
                    t,
                    format!("{:.4} ({:.2}%)", china_traffic, china_percent),
                    format!("{:.4} ({:.2}%)", oversea_traffic, 100.0 - china_percent),
                );
            }
            china_total += china_traffic;
            oversea_total += oversea_traffic;
            if china_traffic + oversea_traffic > 0f64 {
                traffic_rows.push((china_traffic + oversea_traffic, t.to_owned()));
            }
        }
        let mut unit = "MB";
        let mut total = china_total + oversea_total;
        if total > 1024f64 {
            total = total.div(1024f64);
            unit = "GB";
        }
        if !no_print {
            let china_percent = china_total / (china_total + oversea_total) * 100.0;
            println!(
                "{:^20} {:^20} {:^20}",
                "Total".bold().green(),
                format!("{:.4} ({:.2}%)", china_total, china_percent),
                format!("{:.4} ({:.2}%)", oversea_total, 100.0 - china_percent),
            );
        }
        println!(
            "{}{}",
            "流量总计: ".yellow().bold(),
            format!("{:.4}{}", total, unit).bold(),
        );
        // 如果是5分钟的粒度，最后一条有流量的数据量超过流量配置，通过企业微信告警
        // 20240724: 不能只检查最后一条，有可能七牛一次更新2条，而超过流量的出现在倒数第二条
        // 这样会导致漏掉消息，保险起见，一次检查5条
        if granularity == "5min"
            && !traffic_rows.is_empty()
            && !no_warn
            && self.config.monitor.qy_robot.is_some()
        {
            let warn_traffic = self.config.five_minute_traffic.unwrap_or(200) as f64;
            let mut skip = 0;
            if traffic_rows.len() > 5 {
                skip = traffic_rows.len() - 5;
            }
            for (traffic_num, t) in traffic_rows.iter().skip(skip) {
                if traffic_num.ge(&warn_traffic) {
                    let send_mark_file_path = PathBuf::from(
                        format!("/tmp/qiniu/monitor/traffic/{}{}{}", domain, t, traffic_num)
                            .replace(" ", ""),
                    );
                    if send_mark_file_path.exists() {
                        continue;
                    }
                    let msg = format!(
                        "## 🚨七牛CDN流量告警\n\n域名`{}`在`{}` 5分钟内的流量为`{:.4}`MB, 超过告警值`{}`MB，请留意！",
                        domain,
                        t,
                        traffic_num,
                        warn_traffic,
                    );
                    if QyRobot::new(self.config.monitor.qy_robot.clone().unwrap())
                        .send_message(&msg)
                        .await
                        .is_err()
                    {
                        print_err(format!("消息发送失败: {}", msg).as_str(), false);
                    } else {
                        let send_mark_dir = PathBuf::from("/tmp/qiniu/monitor/traffic");
                        if !send_mark_dir.exists() {
                            fs::create_dir_all(send_mark_dir).await?;
                        }
                        File::create(send_mark_file_path).await?.write_i8(1).await?;
                    }
                }
            }
        }
        Ok(china_total + oversea_total)
    }

    pub async fn all_domain_charge_traffic(
        &mut self,
        start_date: &str,
        end_date: &str,
        granularity: &str,
        no_warn: bool,
        domains: Vec<String>,
    ) -> Result<(), anyhow::Error> {
        let mut total = 0.0;
        for d in domains {
            if self.config.debug.unwrap_or(false) {
                println!("[DEBUG] monitor domain: {}", d);
            }
            let response = self
                .charge_traffic(start_date, end_date, granularity, &d)
                .await?;
            println!(
                "域名 {} {}～{} 流量: ",
                d.bold().yellow(),
                start_date.bold(),
                end_date.bold(),
            );
            match self
                .print_traffic_result(
                    &response,
                    true,
                    start_date,
                    end_date,
                    granularity,
                    no_warn,
                    &d,
                )
                .await
            {
                Ok(t) => total += t,
                Err(e) => {
                    print_err(e.to_string().as_str(), false);
                }
            }
        }
        let mut unit = "MB";
        if total > 1024f64 {
            total = total.div(1024f64);
            unit = "GB";
        }
        println!(
            "{}{}",
            "所有域名流量总计: ".red().bold(),
            format!("{:.4}{}", total, unit).bold(),
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::config::Config;

    use super::*;

    #[tokio::test]
    async fn get_charge_traffic_test() {
        let config = Config::parse(Some(PathBuf::from("./qiniu-cdn.toml")));
        let traffic = Client::new(&config, crate::SubFunctionEnum::Traffic);
        let response = traffic
            .charge_traffic("2024-07-14", "2024-07-16", "day", &config.cdn.domain)
            .await
            .unwrap();
        println!("{:#?}", response);
    }
}
