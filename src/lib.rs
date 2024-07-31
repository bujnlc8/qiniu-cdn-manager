use std::fmt::Debug;

use chrono::Local;
use config::Config;

pub mod analysis;
pub mod config;
pub mod domain;
pub mod log;
pub mod prefetch;
pub mod refresh;
pub mod traffic;
pub mod utils;

use anyhow::anyhow;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use utils::token::{ManageTokenGenerator, SignMethod};

// 找不到数据错误提示
const NOT_FOUND_MSG: &str = "未查询到数据！";

// 查询中提示
pub const QUERYING: &str = "查询中，请稍候🔎...";

// 功能分类
#[derive(Debug, PartialEq, PartialOrd)]
pub enum SubFunctionEnum {
    // 计费流量
    Traffic,
    // 刷新缓存
    Refresh,
    // 日志
    Log,
    // 域名相关
    Domain,
    // 文件预取
    Prefetch,
    // TOp
    AnalysisTop,
    // 状态码
    AnalysisStatus,
    // ISP
    AnalysisIsp,
    // 命中率
    AnalysisHitmiss,
    // 请求次数
    AnalysisCount,
}

impl SubFunctionEnum {
    pub fn get_sign_method(&self) -> SignMethod {
        if [Self::Log, Self::Refresh, Self::Prefetch].contains(self) {
            return SignMethod::Method2;
        }
        SignMethod::Method1
    }

    pub fn get_host(&self) -> &str {
        if *self == Self::Domain {
            return "api.qiniu.com";
        }
        "fusion.qiniuapi.com"
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    config: Config,
    host: String,
    sign_method: SignMethod,
}

#[derive(Debug, Deserialize)]
pub struct BaseResponse {
    pub code: i32,
    pub error: String,
}

impl Client {
    pub fn new(config: &Config, sub_func: SubFunctionEnum) -> Self {
        Self {
            config: config.to_owned().clone(),
            host: sub_func.get_host().to_owned(),
            sign_method: sub_func.get_sign_method(),
        }
    }
    /// do request
    #[allow(clippy::too_many_arguments)]
    async fn do_request<T: DeserializeOwned + Debug, D: ?Sized + Serialize + std::marker::Sync>(
        &self,
        method: &str,
        url: &str,
        headers: Option<&reqwest::header::HeaderMap>,
        content_type: Option<&str>,
        data: Option<&D>,
    ) -> Result<T, anyhow::Error> {
        let client = reqwest::Client::new();
        let mut body = "".to_string();
        let body_bytes = match data {
            Some(d) => {
                body = serde_json::to_string(d).unwrap();
                Some(body.as_bytes())
            }
            None => None,
        };
        let mut header = HeaderMap::new();
        if headers.is_some() {
            header = headers.unwrap().clone();
        }
        // 指定content_type
        if !header.contains_key("Content-Type") {
            if let Some(content_type) = content_type {
                header.insert("Content-Type", HeaderValue::from_str(content_type).unwrap());
            } else {
                header.insert(
                    "Content-Type",
                    HeaderValue::from_str("application/json").unwrap(),
                );
            }
        }
        let config = self.config.clone();
        let token_generator =
            ManageTokenGenerator::new(config.cdn.access_key.clone(), config.cdn.secret_key.clone());
        let sign = match self.sign_method {
            SignMethod::Method1 => token_generator.generate_v1(url, content_type, body_bytes)?,
            SignMethod::Method2 => {
                token_generator.generate_v2(method, url, Some(&header), content_type, body_bytes)?
            }
        };
        let authorization = match self.sign_method {
            SignMethod::Method1 => format!("QBox {}", sign),
            SignMethod::Method2 => format!("Qiniu {}", sign),
        };
        header.insert(
            "Authorization",
            HeaderValue::from_str(&authorization).unwrap(),
        );
        let mut builder;
        match method.to_uppercase().as_str() {
            "GET" => builder = client.get(url),
            "POST" => builder = client.post(url),
            "PUT" => builder = client.put(url),
            _ => return Err(anyhow!("不支持该方法: {:?}", method)),
        }
        let mut start = 0;
        if config.debug.unwrap_or(false) {
            println!(
                "[DEBUG] qiniu request: {} {}\n{:#?}\nbody: {}",
                method, url, header, body,
            );
            start = Local::now().timestamp_millis();
        }
        if !body.is_empty() {
            builder = builder.body(body);
        }
        let response = builder.headers(header).send().await?;
        let status = response.status();
        if config.debug.unwrap_or(false) {
            let end = Local::now().timestamp_millis();
            let elapsed = end - start;
            let text = response.text().await;
            if text.is_ok() {
                let text = text.unwrap();
                println!("[DEBUG] qiniu response, {elapsed}ms elapsed: \n{}", text);
                if !status.is_success() {
                    if let Ok(res) = serde_json::from_str::<BaseResponse>(&text) {
                        return Err(anyhow!(
                            "[{}]七牛响应异常，code: {}, error: {}",
                            status.as_u16(),
                            res.code,
                            res.error,
                        ));
                    }
                    return Err(anyhow!("[{}]七牛响应异常", status.as_u16()));
                }
                let r: T = serde_json::from_str(&text).unwrap();
                Ok(r)
            } else {
                Err(anyhow!("七牛响应为空"))
            }
        } else {
            if !status.is_success() {
                if let Ok(res) = response.json::<BaseResponse>().await {
                    return Err(anyhow!(
                        "[{}]七牛响应异常，code: {}, error: {}",
                        status.as_u16(),
                        res.code,
                        res.error,
                    ));
                }
                return Err(anyhow!("[{}]七牛响应异常", status.as_u16()));
            }
            Ok(response.json().await?)
        }
    }
}
