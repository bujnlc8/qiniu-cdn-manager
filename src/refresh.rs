//! 缓存刷新

use std::collections::HashMap;

use anyhow::Ok;
use serde::Deserialize;

use crate::Client;
use colored::Colorize;

#[derive(Debug, Deserialize, Clone)]
pub struct RefreshResponse {
    pub code: i32,
    pub error: String,
    #[serde(rename = "requestId")]
    pub request_id: Option<String>,
    #[serde(rename = "taskIds")]
    pub task_ids: Option<HashMap<String, String>>,
    #[serde(rename = "invalidUrls")]
    pub invalid_urls: Option<Vec<String>>,
    #[serde(rename = "invalidDirs")]
    pub invalid_dirs: Option<Vec<String>>,
    #[serde(rename = "urlQuotaDay")]
    pub url_quota_day: Option<i64>,
    #[serde(rename = "urlSurplusDay")]
    pub url_surplus_day: Option<i64>,
    #[serde(rename = "dirQuotaDay")]
    pub dir_quota_day: Option<i64>,
    #[serde(rename = "dirSurplusDay")]
    pub dir_surplus_day: Option<i64>,
}

impl Client {
    /// ### [刷新CDN缓存](https://developer.qiniu.com/fusion/1229/cache-refresh#3)
    pub async fn refresh(&self, urls: &str, dirs: &str) -> Result<RefreshResponse, anyhow::Error> {
        let url = format!("https://{}{}", self.host, "/v2/tune/refresh");
        let mut url_arr: Vec<&str> = Vec::new();
        if !urls.trim().is_empty() {
            url_arr = urls.split(",").collect();
        }
        let mut dir_arr: Vec<&str> = Vec::new();
        if !dirs.trim().is_empty() {
            dir_arr = dirs.split(',').collect();
        }
        let mut data = HashMap::new();
        data.insert("urls", url_arr);
        data.insert("dirs", dir_arr);
        let response = self
            .do_request::<RefreshResponse, HashMap<&str, Vec<&str>>>(
                "POST",
                &url,
                None,
                Some("application/json"),
                Some(&data),
            )
            .await?;
        Ok(response)
    }

    pub fn print_result(&self, response: &RefreshResponse) {
        if response.code != 200 {
            eprintln!(
                "[ERR] code: {}, message: {}.",
                response.code.to_string().red(),
                response.error.red()
            );
        } else {
            println!("{}", "刷新成功 ✅".green());
        }
        if let Some(invalid_urls) = response.invalid_urls.clone() {
            eprintln!("[ERR] {}", "无效的url列表: ".bold());
            for invalid_url in invalid_urls.iter() {
                eprintln!("{}", invalid_url.yellow());
            }
        }
        if let Some(invalid_dirs) = response.invalid_dirs.clone() {
            eprintln!("[ERR] {}", "无效的url列表: ".bold());
            for invalid_dir in invalid_dirs.iter() {
                eprintln!("{}", invalid_dir.yellow());
            }
        }
        if let Some(url_quota_day) = response.url_quota_day {
            println!(
                "{}{}",
                "每日的刷新url限额: ".bold(),
                url_quota_day.to_string().green()
            );
        }
        if let Some(url_surplus_day) = response.url_surplus_day {
            println!(
                "{}{}",
                "每日的当前剩余的刷新url限额: ".bold(),
                url_surplus_day.to_string().green()
            );
        }
        if let Some(dir_quota_day) = response.dir_quota_day {
            println!(
                "{}{}",
                "每日的刷新dir限额: ".bold(),
                dir_quota_day.to_string().green()
            );
        }
        if let Some(dir_surplus_day) = response.dir_surplus_day {
            println!(
                "{}{}",
                "每日的当前剩余的刷新dir限额: ".bold(),
                dir_surplus_day.to_string().green()
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
    async fn refresh_test() {
        let config = Config::parse(Some(PathBuf::from("./qiniu-cdn.toml")));
        let refresh = Client::new(&config, crate::SubFunctionEnum::Refresh);
        let response = refresh
            .refresh(
                "https://static.example.com/a.json",
                "https://static.example.com/downloads/",
            )
            .await
            .unwrap();
        println!("{:#?}", response)
    }
}
