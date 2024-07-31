//! 区域运营商流量查询
#![allow(clippy::too_many_arguments)]

use std::{collections::HashMap, ops::Div};

use chrono::Local;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use unicode_width::UnicodeWidthStr;

use crate::{
    utils::{
        print_err,
        region_isp::{
            get_isp_name_from_code, get_region_name_from_code, ISP_CODES, REGION_CODE_LIST,
        },
        WaitBlinker,
    },
    Client, NOT_FOUND_MSG,
};

use super::Freq;

#[derive(Debug, Deserialize)]
pub struct ISPTrafficResponse {
    pub code: i32,
    pub error: String,
    pub data: Option<ISPTrafficData>,
}

#[derive(Debug, Deserialize)]
pub struct ISPTrafficData {
    pub points: Option<Vec<String>>,
    pub value: Option<Vec<i64>>,
}

#[derive(Debug, Serialize)]
pub struct ISPTrafficParam<'a> {
    pub domains: Vec<String>,
    pub freq: &'a str,
    pub regions: Vec<&'a str>,
    pub isp: &'a str,
    #[serde(rename = "startDate")]
    pub start_date: &'a str,
    #[serde(rename = "endDate")]
    pub end_date: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct ISPCountResponse {
    pub code: i32,
    pub error: String,
    pub data: Option<ISPCountData>,
}

#[derive(Debug, Deserialize)]
pub struct ISPCountData {
    pub points: Vec<String>,
    #[serde(rename = "ispReq")]
    pub isp_req: HashMap<String, Vec<i64>>,
}

#[derive(Debug, Serialize)]
pub struct ISPCountParam<'a> {
    pub domains: Vec<String>,
    pub freq: &'a str,
    pub region: &'a str,
    #[serde(rename = "startDate")]
    pub start_date: &'a str,
    #[serde(rename = "endDate")]
    pub end_date: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct ISPTrafficRatioResponse {
    pub code: i32,
    pub error: String,
    pub data: Option<HashMap<String, f64>>,
}

#[derive(Debug, Serialize)]
pub struct ISPTrafficRatioParam<'a> {
    pub domains: Vec<String>,
    pub regions: Vec<&'a str>,
    #[serde(rename = "startDate")]
    pub start_date: &'a str,
    #[serde(rename = "endDate")]
    pub end_date: &'a str,
}

impl Client {
    /// ### [区域运营商流量查询](https://developer.qiniu.com/fusion/4081/cdn-log-analysis#4)
    pub async fn isp_traffic(
        self,
        freq: Freq,
        regions: &str,
        isp: &str,
        start_date: &str,
        end_date: &str,
        domains: Vec<String>,
    ) -> Result<ISPTrafficResponse, anyhow::Error> {
        let url = format!("https://{}{}", self.host, "/v2/tune/loganalyze/traffic");
        let freq = freq.to_string();
        let data = ISPTrafficParam {
            domains,
            freq: freq.as_str(),
            regions: regions.split(',').collect(),
            isp,
            start_date,
            end_date,
        };
        let response = self
            .do_request::<ISPTrafficResponse, ISPTrafficParam>(
                "POST",
                &url,
                None,
                Some("application/json"),
                Some(&data),
            )
            .await?;
        Ok(response)
    }

    pub fn print_isp_traffic(
        &self,
        response: ISPTrafficResponse,
        limit: Option<i32>,
        isp: &str,
        regions: &str,
        start_date: &str,
        end_date: &str,
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
        if data.points.is_none() {
            print_err(NOT_FOUND_MSG, true);
        }
        let limit = limit.unwrap_or(10000);
        let now = Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
        let mut total = 0f64;
        println!(
            "域名 {} 运营商 {} 区域 {} {}～{}流量如下: ",
            domains.join(",").yellow().bold(),
            isp.bold().yellow(),
            regions.bold().yellow(),
            start_date.bold(),
            end_date.bold(),
        );
        println!(
            "{:^20} {:^20}",
            "Time".green().bold(),
            "Traffic(MB)".green().bold()
        );
        let values = data.value.unwrap();
        for (i, t) in data.points.unwrap().iter().enumerate() {
            let traffic = match values.get(i) {
                Some(v) => (*v as f64).div(1024.0).div(1024.0),
                None => 0f64,
            };
            if traffic == 0f64 && now < *t {
                continue;
            }
            total += traffic;
            println!("{:^20} {:^20}", t, format!("{:.4}", traffic),);
            if (i + 1) as i32 >= limit {
                break;
            }
        }
        let mut unit = "";
        if total > 1024f64 {
            total /= 1024f64;
            unit = "GB";
        }
        if total > 0f64 {
            println!(
                "{:^20} {:^20}",
                "Total".green().bold(),
                format!("{:.4}{}", total, unit),
            );
        }
    }

    /// ### [查询 ISP 请求次数](https://developer.qiniu.com/fusion/4081/cdn-log-analysis#9)
    pub async fn isp_count(
        &self,
        freq: Freq,
        region: &str,
        start_date: &str,
        end_date: &str,
        domains: Vec<String>,
    ) -> Result<ISPCountResponse, anyhow::Error> {
        let url = format!("https://{}{}", self.host, "/v2/tune/loganalyze/ispreqcount");
        let freq = freq.to_string();
        let data = ISPCountParam {
            domains,
            freq: freq.as_str(),
            region,
            start_date,
            end_date,
        };
        let response = self
            .do_request::<ISPCountResponse, ISPCountParam>(
                "POST",
                &url,
                None,
                Some("application/json"),
                Some(&data),
            )
            .await?;
        Ok(response)
    }

    pub fn print_isp_count(
        &self,
        response: ISPCountResponse,
        limit: Option<i32>,
        region: &str,
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
        let isps = data.isp_req;
        let mut isp_codes: Vec<&String> = isps.keys().collect();
        let others = "others".to_string();
        if isp_codes.contains(&&others) {
            for (i, v) in isp_codes.iter().enumerate() {
                if *v == "others" {
                    isp_codes.remove(i);
                    break;
                }
            }
            isp_codes.sort();
            isp_codes.push(&others);
        } else {
            isp_codes.sort();
        }
        let limit = limit.unwrap_or(10000);
        let mut status_map: HashMap<&String, i64> = HashMap::new();
        let mut total = 0;
        println!(
            "域名 {} 区域 {} {}～{}请求次数如下: ",
            domains.join(",").yellow().bold(),
            region.bold().yellow(),
            start_date.bold(),
            end_date.bold(),
        );
        print!("{:^20}", "Time".green().bold());
        for isp_code in isp_codes.iter() {
            print!("{:^20}", isp_code.green().bold());
        }
        println!();
        for (i, t) in data.points.iter().enumerate() {
            print!("{:^20}", t);
            let mut time_total = 0;
            for status_code in isp_codes.iter() {
                let tmp = isps.get(*status_code).unwrap();
                let count = tmp.get(i).unwrap();
                time_total += count;
            }
            for status_code in isp_codes.iter() {
                let tmp = isps.get(*status_code).unwrap();
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
            for status_code in isp_codes.iter() {
                print!("{:^20}", status_map.get(*status_code).unwrap_or(&0i64));
            }
            println!();
            print!("{:^20}", "Percent".green().bold());
            for status_code in isp_codes.iter() {
                let status_count = *status_map.get(*status_code).unwrap_or(&0i64);
                print!(
                    "{:^20}",
                    format!("{:.2}%", (status_count as f64).div(total as f64) * 100f64)
                );
            }
            println!();
        }
    }

    /// ### [查询 ISP 流量占比](https://developer.qiniu.com/fusion/4081/cdn-log-analysis#10)
    pub async fn isp_traffic_ratio(
        &self,
        regions: &str,
        start_date: &str,
        end_date: &str,
        domains: Vec<String>,
    ) -> Result<ISPTrafficRatioResponse, anyhow::Error> {
        let url = format!("https://{}{}", self.host, "/v2/tune/loganalyze/isptraffic");
        let data = ISPTrafficRatioParam {
            domains,
            regions: regions.split(',').collect(),
            start_date,
            end_date,
        };
        let response = self
            .do_request::<ISPTrafficRatioResponse, ISPTrafficRatioParam>(
                "POST",
                &url,
                None,
                Some("application/json"),
                Some(&data),
            )
            .await?;
        Ok(response)
    }

    pub fn print_traffic_ratio(
        &self,
        response: ISPTrafficRatioResponse,
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
        let mut isp_codes: Vec<&String> = data.keys().collect();
        if isp_codes.is_empty() {
            print_err(NOT_FOUND_MSG, true);
        }
        println!(
            "域名 {} 区域 {} {}～{} 流量占比如下: ",
            domains.join(",").yellow().bold(),
            regions.bold().yellow(),
            start_date.bold(),
            end_date.bold(),
        );
        println!(
            "{:<10} {:^10}",
            "运营商".bold().green(),
            "Percent(%)".bold().green()
        );
        let others = "others".to_string();
        if isp_codes.contains(&&others) {
            for (i, v) in isp_codes.iter().enumerate() {
                if *v == "others" {
                    isp_codes.remove(i);
                    break;
                }
            }
            isp_codes.sort();
            isp_codes.push(&others);
        } else {
            isp_codes.sort();
        }
        for isp_code in isp_codes.iter() {
            println!(
                "{:<10} {:^10}",
                get_isp_name_from_code(isp_code),
                format!("{:.2}%", data.get(*isp_code).unwrap(),)
            );
        }
    }

    pub async fn isp_traffic_sort(
        &self,
        freq: Freq,
        regions: &str,
        isp: &str,
        start_date: &str,
        end_date: &str,
        domains: Vec<String>,
        wait_blink: Option<WaitBlinker>,
        isp_sort: bool,
    ) -> Result<(), anyhow::Error> {
        let (tx, mut rx) = mpsc::channel(40);
        let iter_variables = if isp_sort {
            ISP_CODES.to_vec()
        } else {
            REGION_CODE_LIST.to_vec()
        };
        for val in iter_variables {
            if ["global", "china", "all"].contains(&val) {
                continue;
            }
            let tx1 = tx.clone();
            let region = if isp_sort {
                regions.to_string()
            } else {
                val.to_string()
            };
            let isp = if isp_sort {
                val.to_string()
            } else {
                isp.to_string()
            };
            let start_date = start_date.to_string();
            let end_date = end_date.to_string();
            let domains = domains.clone();
            let this = self.clone();
            tokio::spawn(async move {
                let mut data = HashMap::new();
                let response = this
                    .isp_traffic(freq, &region, &isp, &start_date, &end_date, domains)
                    .await
                    .unwrap();
                let mut total = 0;
                if let Some(data) = response.data {
                    if let Some(data) = data.value {
                        total = data.iter().sum();
                    }
                }
                if isp_sort {
                    data.insert(isp, total);
                } else {
                    data.insert(region, total);
                }
                tx1.send(data).await.unwrap();
                drop(tx1);
            });
        }
        drop(tx);
        let mut data_map = HashMap::new();
        while let Some(data) = rx.recv().await {
            data_map.extend(data.clone());
        }
        let mut vec: Vec<(String, i64)> = data_map.into_iter().collect();
        let total: i64 = vec.iter().map(|x| x.1).collect::<Vec<i64>>().iter().sum();
        vec.sort_by(|a, b| b.1.cmp(&a.1));
        if let Some(blinker) = wait_blink {
            blinker.sender.send(true).unwrap();
            blinker.handle.await?;
        }
        let sort_name = if !isp_sort { "区域" } else { "运营商" };
        let filter_name = if isp_sort { "区域" } else { "运营商" };
        let filter_value = if isp_sort { regions } else { isp };
        println!(
            "域名 {} {} {} {}～{} 流量{}分布: ",
            domains.join(",").yellow().bold(),
            filter_name,
            filter_value.yellow().bold(),
            start_date.bold(),
            end_date.bold(),
            sort_name,
        );
        println!(
            "{:<20} {:^20} {:^20}",
            sort_name.green().bold(),
            "Traffic(MB)".green().bold(),
            "Percent(%)".green().bold()
        );
        for (k, v) in vec {
            let mut traffic = v as f64 / 1024.0 / 1024.0;
            let mut unit = "";
            if traffic > 1024.0 {
                traffic /= 1024.0;
                unit = "GB";
            }
            let name = if !isp_sort {
                get_region_name_from_code(&k)
            } else {
                get_isp_name_from_code(&k)
            };
            println!(
                "{}{} {:^20} {:^20}",
                name,
                " ".repeat(20 - UnicodeWidthStr::width_cjk(name)),
                format!("{:.4}{}", traffic, unit),
                format!("{:.2}%", v as f64 / total as f64 * 100.0)
            );
        }
        if total > 0 {
            println!(
                "{}{} {:^20} {:^20}",
                "Total".green().bold(),
                " ".repeat(15),
                format!("{:.4}GB", total as f64 / 1024.0 / 1024.0 / 1024.0)
                    .as_str()
                    .bold(),
                "100%".bold(),
            );
        }
        Ok(())
    }

    pub async fn isp_count_all_region(
        &self,
        freq: Freq,
        start_date: &str,
        end_date: &str,
        domains: Vec<String>,
        wait_blink: Option<WaitBlinker>,
    ) -> Result<(), anyhow::Error> {
        let (tx, mut rx) = mpsc::channel(40);
        for region in REGION_CODE_LIST {
            if ["global", "china"].contains(&region) {
                continue;
            }
            let tx1 = tx.clone();
            let region = region.to_string();
            let start_date = start_date.to_string();
            let end_date = end_date.to_string();
            let domains = domains.clone();
            let this = self.clone();
            tokio::spawn(async move {
                let mut data = HashMap::new();
                let response = this
                    .isp_count(freq, &region, &start_date, &end_date, domains)
                    .await
                    .unwrap();
                let mut total = 0;
                if let Some(data) = response.data {
                    total = data.isp_req.values().fold(0i64, |mut acc, v| {
                        acc += v.iter().sum::<i64>();
                        acc
                    });
                }
                data.insert(region, total);
                tx1.send(data).await.unwrap();
                drop(tx1);
            });
        }
        drop(tx);
        let mut data_map = HashMap::new();
        while let Some(data) = rx.recv().await {
            data_map.extend(data);
        }
        let mut vec: Vec<(String, i64)> = data_map.into_iter().collect();
        let total: i64 = vec.iter().map(|x| x.1).collect::<Vec<i64>>().iter().sum();
        vec.sort_by(|a, b| b.1.cmp(&a.1));
        if let Some(blinker) = wait_blink {
            blinker.sender.send(true).unwrap();
            blinker.handle.await?;
        }
        println!(
            "域名 {} {}～{} 请求次数地区分布: ",
            domains.join(",").yellow().bold(),
            start_date.bold(),
            end_date.bold(),
        );
        println!(
            "{:<20} {:^20} {:^20}",
            "Region".green().bold(),
            "Count".green().bold(),
            "Percent(%)".green().bold()
        );
        for (k, v) in vec {
            let region_name = get_region_name_from_code(&k);
            println!(
                "{}{} {:^20} {:^20}",
                region_name,
                " ".repeat(20 - UnicodeWidthStr::width_cjk(region_name)),
                v,
                format!("{:.2}%", v as f64 / total as f64 * 100.0)
            );
        }
        if total > 0 {
            println!(
                "{}{} {:^20} {:^20}",
                "Total".green().bold(),
                " ".repeat(15),
                total.to_string().bold(),
                "100%".bold(),
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::config::Config;

    use super::*;

    #[tokio::test]
    async fn isp_traffic_test() {
        let config = Config::parse(Some(PathBuf::from("./qiniu-cdn.toml")));
        let analysis = Client::new(&config, crate::SubFunctionEnum::AnalysisIsp);
        let response = analysis
            .isp_traffic(
                Freq::OneDay,
                "china",
                "unicom",
                "2024-07-16",
                "2024-07-16",
                vec![config.cdn.domain.clone()],
            )
            .await
            .unwrap();
        println!("{:#?}", response)
    }
}
