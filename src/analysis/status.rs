//! 查询状态码
#![allow(clippy::too_many_arguments)]

use std::{collections::HashMap, ops::Div};

use anyhow::Ok;
use colored::Colorize;
use serde::{Deserialize, Serialize};

use super::Freq;
use crate::{utils::print_err, Client, NOT_FOUND_MSG};

#[derive(Debug, Deserialize)]
pub struct StatusResponse {
    pub code: i32,
    pub error: String,
    pub data: Option<StatusData>,
}

#[derive(Debug, Deserialize)]
pub struct StatusData {
    pub points: Vec<String>,
    pub codes: HashMap<String, Vec<i64>>,
}

#[derive(Debug, Serialize)]
pub struct StatusParam<'a> {
    pub domains: Vec<String>,
    pub freq: &'a str,
    pub regions: Vec<&'a str>,
    pub isp: &'a str,
    #[serde(rename = "startDate")]
    pub start_date: &'a str,
    #[serde(rename = "endDate")]
    pub end_date: &'a str,
}

impl Client {
    /// ### [查询状态码](https://developer.qiniu.com/fusion/4081/cdn-log-analysis#6)
    pub async fn status_code(
        &self,
        freq: Freq,
        regions: &str,
        isp: &str,
        start_date: &str,
        end_date: &str,
        domains: Vec<String>,
    ) -> Result<StatusResponse, anyhow::Error> {
        let url = format!("https://{}{}", self.host, "/v2/tune/loganalyze/statuscode");
        let freq = freq.to_string();
        let data = StatusParam {
            domains,
            freq: freq.as_str(),
            regions: regions.split(',').collect(),
            isp,
            start_date,
            end_date,
        };
        let response = self
            .do_request::<StatusResponse, StatusParam>(
                "POST",
                &url,
                None,
                Some("application/json"),
                Some(&data),
            )
            .await?;
        Ok(response)
    }

    pub fn print_status(
        &self,
        response: StatusResponse,
        limit: Option<i32>,
        isp: &str,
        regions: &str,
        start_date: &str,
        end_date: &str,
        domains: Vec<String>,
    ) {
        if response.code != 200 {
            print_err(
                format!("code: {}, message: {}.", response.code, response.error).as_str(),
                true,
            );
        }
        if response.data.is_none() {
            print_err(NOT_FOUND_MSG, true);
        }
        let data = response.data.unwrap();
        let codes = data.codes;
        let mut status_codes: Vec<&String> = codes.keys().collect();
        status_codes.sort();
        let limit = limit.unwrap_or(10000);
        let mut status_map: HashMap<&String, i64> = HashMap::new();
        let mut total = 0;
        println!(
            "域名 {} 运营商 {} 区域 {} {}～{}状态码如下: ",
            domains.join(",").yellow().bold(),
            isp.bold().yellow(),
            regions.bold().yellow(),
            start_date.bold(),
            end_date.bold(),
        );
        print!("{:^20}", "Time".green().bold());
        for status_code in status_codes.iter() {
            print!("{:^20}", status_code.green().bold());
        }
        println!();
        for (i, t) in data.points.iter().enumerate() {
            print!("{:^20}", t);
            let mut time_total = 0;
            for status_code in status_codes.iter() {
                let tmp = codes.get(*status_code).unwrap();
                let count = tmp.get(i).unwrap();
                time_total += count;
            }
            for status_code in status_codes.iter() {
                let tmp = codes.get(*status_code).unwrap();
                let count = tmp.get(i).unwrap();
                total += count;
                print!(
                    "{:^20}",
                    format!(
                        "{} ({:.2}%)",
                        count,
                        (*count as f64 / time_total as f64) * 100f64
                    )
                );
                let exist = status_map.get(*status_code);
                if let Some(exist) = exist {
                    status_map.insert(status_code, exist + count);
                } else {
                    status_map.insert(status_code, *count);
                }
            }
            println!();
            if (i + 1) as i32 >= limit {
                break;
            }
        }
        if total > 0 {
            print!("{:^20}", "Total".green().bold());
            for status_code in status_codes.iter() {
                print!("{:^20}", status_map.get(*status_code).unwrap_or(&0i64));
            }
            println!();
            print!("{:^20}", "Percent".green().bold());
            for status_code in status_codes.iter() {
                let status_count = *status_map.get(*status_code).unwrap_or(&0i64);
                print!(
                    "{:^20}",
                    format!("{:.2}%", (status_count as f64).div(total as f64) * 100f64)
                );
            }
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::config::Config;

    use super::*;

    #[tokio::test]
    async fn status_test() {
        let config = Config::parse(Some(PathBuf::from("./qiniu-cdn.toml")));
        let analysis = Client::new(&config, crate::SubFunctionEnum::AnalysisStatus);
        let response = analysis
            .status_code(
                Freq::OneDay,
                "china",
                "unicom",
                "2024-07-15",
                "2024-07-16",
                vec![config.cdn.domain.clone()],
            )
            .await
            .unwrap();
        println!("{:#?}", response)
    }
}
