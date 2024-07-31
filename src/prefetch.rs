//! 文件预取
use std::collections::HashMap;

use serde::Deserialize;

use crate::Client;

use colored::Colorize;

#[derive(Debug, Deserialize)]
pub struct PrefetchResponse {
    pub code: i32,
    pub error: String,
    #[serde(rename = "requestId")]
    pub request_id: Option<String>,
    #[serde(rename = "invalidUrls")]
    pub invalid_urls: Option<Vec<String>>,
    #[serde(rename = "quotaDay")]
    pub quota_day: Option<i32>,
    #[serde(rename = "surplusDay")]
    pub surplus_day: Option<i32>,
}

impl Client {
    /// ### [预取](https://developer.qiniu.com/fusion/1227/file-prefetching#3)
    pub async fn prefetch(&self, urls: &str) -> Result<PrefetchResponse, anyhow::Error> {
        let url = format!("https://{}{}", self.host, "/v2/tune/prefetch");
        let url_arr: Vec<&str> = urls.split(',').collect();
        let mut data = HashMap::new();
        data.insert("urls", url_arr);
        let response = self
            .do_request::<PrefetchResponse, HashMap<&str, Vec<&str>>>(
                "POST",
                &url,
                None,
                Some("application/json"),
                Some(&data),
            )
            .await?;
        Ok(response)
    }

    pub fn print_prefetch(self, response: PrefetchResponse) {
        if response.code != 200 {
            eprintln!(
                "[ERR] code: {}, message: {}.",
                response.code.to_string().red(),
                response.error.red()
            );
        } else {
            println!("{}", "文件预取成功 ✅".green());
        }
        if let Some(request_id) = response.request_id {
            println!("{}: {}", "RequestId".green().bold(), request_id);
        }
        if let Some(invalid_urls) = response.invalid_urls {
            println!("{}: ", "存在如下无效的 url".green().bold());
            for invalid_url in invalid_urls {
                println!("{}", invalid_url);
            }
        }
        if let Some(quota_day) = response.quota_day {
            println!("{}: {}", "每日预取限额".green().bold(), quota_day);
        }
        if let Some(surplus_day) = response.surplus_day {
            println!("{}: {}", "当前剩余限额".green().bold(), surplus_day);
        }
    }
}
