//! 查询命中率

use std::ops::Div;

use anyhow::Ok;
use chrono::Local;
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{utils::print_err, Client, NOT_FOUND_MSG};

use super::Freq;

#[derive(Debug, Deserialize)]
pub struct HitMissResponse {
    pub code: i32,
    pub error: String,
    pub data: Option<HitMissData>,
}

#[derive(Debug, Deserialize)]
pub struct HitMissData {
    pub points: Vec<String>,
    pub hit: Vec<i64>,
    pub miss: Vec<i64>,
    #[serde(rename = "trafficHit")]
    pub traffic_hit: Vec<i64>,
    #[serde(rename = "trafficMiss")]
    pub traffic_miss: Vec<i64>,
}

#[derive(Debug, Serialize)]
pub struct HitMissParam<'a> {
    pub domains: Vec<String>,
    pub freq: &'a str,
    #[serde(rename = "startDate")]
    pub start_date: &'a str,
    #[serde(rename = "endDate")]
    pub end_date: &'a str,
}

impl Client {
    /// ### [查询命中率](https://developer.qiniu.com/fusion/4081/cdn-log-analysis#7)
    pub async fn hit_miss(
        &self,
        freq: Freq,
        start_date: &str,
        end_date: &str,
        domains: Vec<String>,
    ) -> Result<HitMissResponse, anyhow::Error> {
        let url = format!("https://{}{}", self.host, "/v2/tune/loganalyze/hitmiss");
        let freq = freq.to_string();
        let data = HitMissParam {
            domains,
            freq: freq.as_str(),
            start_date,
            end_date,
        };
        let response = self
            .do_request::<HitMissResponse, HitMissParam>(
                "POST",
                &url,
                None,
                Some("application/json"),
                Some(&data),
            )
            .await?;
        Ok(response)
    }

    pub fn print_hitmiss(
        &self,
        response: HitMissResponse,
        limit: Option<i32>,
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
        let limit = limit.unwrap_or(10000);
        let now = Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
        println!(
            "域名 {} {}～{}命中率如下: ",
            domains.join(",").yellow().bold(),
            start_date.bold(),
            end_date.bold(),
        );
        println!(
            "{:^20} {:^20} {:^20} {:^20} {:^20}",
            "Time".bold().green(),
            "HitCount".bold().green(),
            "MissCount".bold().green(),
            "HitTraffic(MB)".bold().green(),
            "MissTraffic(MB)".bold().green(),
        );
        let mut hit_total = 0;
        let mut miss_total = 0;
        let mut hit_traffic_total = 0f64;
        let mut miss_traffic_total = 0f64;
        for (i, t) in data.points.iter().enumerate() {
            let mut hit_traffic = *data.traffic_hit.get(i).unwrap() as f64;
            hit_traffic = hit_traffic.div(1024f64).div(1024f64);
            let mut miss_traffic = *data.traffic_miss.get(i).unwrap() as f64;
            miss_traffic = miss_traffic.div(1024f64).div(1024f64);
            let hit = data.hit.get(i).unwrap();
            let miss = data.miss.get(i).unwrap();
            hit_total += hit;
            miss_total += miss;
            hit_traffic_total += hit_traffic;
            miss_traffic_total += miss_traffic;
            if now < *t
                && hit_traffic == 0f64
                && miss_traffic == 0f64
                && *hit == 0i64
                && *miss == 0i64
            {
                continue;
            }
            let hit_rate = (*hit as f64) / ((hit + miss) as f64) * 100f64;
            let miss_rate = 100f64 - hit_rate;
            let hit_traffic_rate = hit_traffic / (hit_traffic + miss_traffic) * 100f64;
            let miss_traffic_rate = 100f64 - hit_traffic_rate;
            println!(
                "{:^20} {:^20} {:^20} {:^20} {:^20}",
                t,
                format!("{} ({:.2}%)", hit, hit_rate),
                format!("{} ({:.2}%)", miss, miss_rate),
                format!("{:.4} ({:.2}%)", hit_traffic, hit_traffic_rate),
                format!("{:.4} ({:.2}%)", miss_traffic, miss_traffic_rate),
            );
            if (i + 1) as i32 >= limit {
                break;
            }
        }
        if hit_total > 0 {
            println!(
                "{:^20} {:^20} {:^20} {:^20} {:^20}",
                "Total".bold().green(),
                hit_total,
                miss_total,
                format!("{:.4}", hit_traffic_total),
                format!("{:.4}", miss_traffic_total),
            );
            let hit_rate = (hit_total as f64) / ((hit_total + miss_total) as f64) * 100f64;
            let miss_rate = 100f64 - hit_rate;
            let hit_traffic_rate =
                hit_traffic_total / (hit_traffic_total + miss_traffic_total) * 100f64;
            let miss_traffic_rate = 100f64 - hit_traffic_rate;
            println!(
                "{:^20} {:^20} {:^20} {:^20} {:^20}",
                "Percent".bold().green(),
                format!("{:.2}%", hit_rate),
                format!("{:.2}%", miss_rate),
                format!("{:.2}%", hit_traffic_rate),
                format!("{:.2}%", miss_traffic_rate),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::config::Config;

    use super::*;

    #[tokio::test]
    async fn hit_miss_test() {
        let config = Config::parse(Some(PathBuf::from("./qiniu-cdn.toml")));
        let analysis = Client::new(&config, crate::SubFunctionEnum::AnalysisHitmiss);
        let response = analysis
            .hit_miss(
                Freq::OneDay,
                "2024-07-16",
                "2024-07-16",
                vec![config.cdn.domain.clone()],
            )
            .await
            .unwrap();
        println!("{:#?}", response)
    }
}
