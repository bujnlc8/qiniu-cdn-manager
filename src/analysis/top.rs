//! 请求访问次数及流量 Top IP | URL
#![allow(clippy::too_many_arguments)]

use std::{fmt::Debug, ops::Div};

use anyhow::Ok;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    utils::{max_length, print_err},
    Client, NOT_FOUND_MSG,
};
use colored::Colorize;

#[derive(Debug, Deserialize)]
pub struct TopIpResponse {
    pub code: i32,
    pub error: String,
    pub data: Option<TopIpData>,
}

#[derive(Debug, Deserialize)]
pub struct TopIpData {
    pub ips: Option<Vec<String>>,
    pub count: Option<Vec<i64>>,
    pub traffic: Option<Vec<i64>>,
}

#[derive(Debug, Deserialize)]
pub struct TopUrlResponse {
    pub code: i32,
    pub error: String,
    pub data: Option<TopUrlData>,
}

#[derive(Debug, Deserialize)]
pub struct TopUrlData {
    pub urls: Option<Vec<String>>,
    pub count: Option<Vec<i64>>,
    pub traffic: Option<Vec<i64>>,
}

#[derive(Debug, Serialize)]
pub struct TopParam<'a> {
    pub domains: Vec<String>,
    pub region: &'a str,
    #[serde(rename = "startDate")]
    pub start_date: &'a str,
    #[serde(rename = "endDate")]
    pub end_date: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum FilterType {
    ReqCount,
    Traffic,
}

impl Client {
    async fn top_func<T: DeserializeOwned + Debug>(
        &self,
        region: &str,
        start_date: &str,
        end_date: &str,
        filter_type: FilterType,
        ip_or_url: &str,
        domains: Vec<String>,
    ) -> Result<T, anyhow::Error> {
        let url = match filter_type {
            FilterType::Traffic => format!(
                "https://{}{}{}",
                self.host, "/v2/tune/loganalyze/toptraffic", ip_or_url,
            ),
            FilterType::ReqCount => {
                format!(
                    "https://{}{}{}",
                    self.host, "/v2/tune/loganalyze/topcount", ip_or_url,
                )
            }
        };
        let data = TopParam {
            domains,
            region,
            start_date,
            end_date,
        };
        let response = self
            .do_request::<T, TopParam>("POST", &url, None, Some("application/json"), Some(&data))
            .await?;
        Ok(response)
    }

    /// ## 请求量及数据量Top IP
    /// ### [请求访问次数 Top IP](https://developer.qiniu.com/fusion/4081/cdn-log-analysis#11)
    /// ### [请求访问流量 Top IP](https://developer.qiniu.com/fusion/4081/cdn-log-analysis#12)
    pub async fn top_ip(
        &self,
        region: &str,
        start_date: &str,
        end_date: &str,
        filter_type: FilterType,
        domains: Vec<String>,
    ) -> Result<TopIpResponse, anyhow::Error> {
        self.top_func::<TopIpResponse>(region, start_date, end_date, filter_type, "ip", domains)
            .await
    }
    /// ## 请求量及数据量Top URL
    /// ### [请求访问次数 Top URL](https://developer.qiniu.com/fusion/4081/cdn-log-analysis#14)
    /// ### [请求访问流量 Top URL](https://developer.qiniu.com/fusion/4081/cdn-log-analysis#13)
    pub async fn top_url(
        &self,
        region: &str,
        start_date: &str,
        end_date: &str,
        filter_type: FilterType,
        domains: Vec<String>,
    ) -> Result<TopUrlResponse, anyhow::Error> {
        self.top_func::<TopUrlResponse>(region, start_date, end_date, filter_type, "url", domains)
            .await
    }

    pub fn print_top_ip(
        &self,
        response: TopIpResponse,
        filter_type: FilterType,
        limit: Option<i32>,
        start_date: &str,
        end_date: &str,
        region: &str,
        domains: Vec<String>,
    ) {
        if response.code != 200 {
            let msg = format!("code: {}, message: {}", response.code, response.error);
            print_err(msg.as_str(), true);
        }
        if response.data.is_none() {
            print_err(NOT_FOUND_MSG, true);
        }
        let data = response.data.unwrap();
        let limit = limit.unwrap_or(10000);
        match filter_type {
            FilterType::Traffic => {
                if data.traffic.is_none() || data.traffic.clone().unwrap().is_empty() {
                    print_err(NOT_FOUND_MSG, false)
                } else {
                    let width = max_length(&data.ips.clone().unwrap(), limit);
                    println!(
                        "域名 {} 区域 {} {}～{} Top流量IP如下: ",
                        domains.join(",").yellow().bold(),
                        region.bold().yellow(),
                        start_date.bold(),
                        end_date.bold(),
                    );
                    println!(
                        "{:^width$} {:^20}",
                        "IP".bold().green(),
                        "Traffic(MB)".bold().green(),
                        width = width,
                    );
                    let traffic = data.traffic.unwrap();
                    for (i, ip) in data.ips.unwrap().iter().enumerate() {
                        let traffic_ = match traffic.get(i) {
                            Some(v) => (*v as f64).div(1024.0).div(1024.0),
                            None => 0f64,
                        };
                        println!(
                            "{:^width$} {:^20}",
                            ip,
                            format!("{:.4}", traffic_),
                            width = width
                        );
                        if (i + 1) as i32 >= limit {
                            break;
                        }
                    }
                }
            }
            FilterType::ReqCount => {
                if data.count.is_none() || data.count.clone().unwrap().is_empty() {
                    print_err(NOT_FOUND_MSG, false)
                } else {
                    let width = max_length(&data.ips.clone().unwrap(), limit);
                    println!(
                        "域名 {} 区域 {} {}～{} Top请求次数IP如下: ",
                        domains.join(",").yellow().bold(),
                        region.bold().yellow(),
                        start_date.bold(),
                        end_date.bold(),
                    );
                    println!(
                        "{:^width$} {:^20}",
                        "IP".bold().green(),
                        "Count".bold().green(),
                        width = width,
                    );
                    let count = data.count.unwrap();
                    for (i, ip) in data.ips.unwrap().iter().enumerate() {
                        let count_ = match count.get(i) {
                            Some(v) => *v,
                            None => 0i64,
                        };
                        println!("{:^width$} {:^20}", ip, count_, width = width);
                        if (i + 1) as i32 >= limit {
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn print_top_url(
        &self,
        response: TopUrlResponse,
        filter_type: FilterType,
        limit: Option<i32>,
        start_date: &str,
        end_date: &str,
        region: &str,
        domains: Vec<String>,
    ) {
        if response.code != 200 {
            let msg = format!("code: {}, message: {}", response.code, response.error);
            print_err(msg.as_str(), true)
        }
        if response.data.is_none() {
            print_err(NOT_FOUND_MSG, true)
        }
        let data = response.data.unwrap();
        let limit = limit.unwrap_or(10000);
        match filter_type {
            FilterType::Traffic => {
                if data.traffic.is_none() || data.traffic.clone().unwrap().is_empty() {
                    print_err(NOT_FOUND_MSG, false)
                } else {
                    let width = max_length(&data.urls.clone().unwrap(), limit);
                    println!(
                        "域名 {} 区域 {} {}～{} Top流量URL如下: ",
                        domains.join(",").yellow().bold(),
                        region.bold().yellow(),
                        start_date.bold(),
                        end_date.bold(),
                    );
                    println!(
                        "{:^width$} {:^20}",
                        "URL".bold().green(),
                        "Traffic(MB)".bold().green(),
                        width = width,
                    );
                    let traffic = data.traffic.unwrap();
                    for (i, url) in data.urls.unwrap().iter().enumerate() {
                        let traffic_ = match traffic.get(i) {
                            Some(v) => (*v as f64).div(1024.0).div(1024.0),
                            None => 0f64,
                        };
                        println!(
                            "{:<width$} {:^20}",
                            url,
                            format!("{:.4}", traffic_),
                            width = width
                        );
                        if (i + 1) as i32 >= limit {
                            break;
                        }
                    }
                }
            }
            FilterType::ReqCount => {
                if data.count.is_none() || data.count.clone().unwrap().is_empty() {
                    print_err(NOT_FOUND_MSG, false)
                } else {
                    let width = max_length(&data.urls.clone().unwrap(), limit);
                    println!(
                        "域名 {} 区域 {} {}～{} Top请求次数URL如下: ",
                        domains.join(",").yellow().bold(),
                        region.bold().yellow(),
                        start_date.bold(),
                        end_date.bold(),
                    );
                    println!(
                        "{:^width$} {:^20}",
                        "URL".bold().green(),
                        "Count".bold().green(),
                        width = width,
                    );
                    let count = data.count.unwrap();
                    for (i, url) in data.urls.unwrap().iter().enumerate() {
                        let count_ = match count.get(i) {
                            Some(v) => *v,
                            None => 0i64,
                        };
                        println!("{:<width$} {:^20}", url, count_, width = width);
                        if (i + 1) as i32 >= limit {
                            break;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::config::Config;

    use super::*;

    #[tokio::test]
    async fn top_ip_test() {
        let config = Config::parse(Some(PathBuf::from("./qiniu-cdn.toml")));
        let analysis = Client::new(&config, crate::SubFunctionEnum::AnalysisTop);
        let domains = vec![config.cdn.domain.clone()];
        let response = analysis
            .top_ip(
                "global",
                "2024-07-16",
                "2024-07-16",
                FilterType::Traffic,
                domains.clone(),
            )
            .await
            .unwrap();
        println!("{:#?}", response);
        let response = analysis
            .top_url(
                "global",
                "2024-07-16",
                "2024-07-16",
                FilterType::Traffic,
                domains,
            )
            .await
            .unwrap();
        println!("{:#?}", response)
    }
}
