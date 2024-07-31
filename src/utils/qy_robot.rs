//! 企业微信

use std::collections::HashMap;

use anyhow::{anyhow, Ok};
use reqwest::{header::HeaderValue, Response};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
pub struct QyRobot {
    pub url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct QyMsg<'a> {
    pub msgtype: &'a str,
    pub markdown: HashMap<&'a str, &'a str>,
}

impl QyRobot {
    pub fn new(url: String) -> Self {
        QyRobot { url: Some(url) }
    }

    pub async fn send_message(&self, message: &str) -> Result<Response, anyhow::Error> {
        if self.url.is_none() {
            return Err(anyhow!("未找到企业微信机器人链接"));
        }
        let mut map = HashMap::new();
        map.insert("content", message);
        let body = QyMsg {
            msgtype: "markdown",
            markdown: map,
        };
        let response = reqwest::Client::new()
            .post(self.url.clone().unwrap())
            .header(
                "Content-Type",
                HeaderValue::from_str("application/json").unwrap(),
            )
            .json(&body)
            .send()
            .await?;
        Ok(response)
    }
}
