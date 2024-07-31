//! 管理凭证
use std::collections::HashMap;

use anyhow::{Error, Ok};
use base64::prelude::*;
use reqwest::Url;

pub struct ManageTokenGenerator {
    access_key: String,
    secret_key: String,
}

/// 签名方法类型
#[derive(Debug, Clone, Copy)]
pub enum SignMethod {
    Method1,
    Method2,
}

impl ManageTokenGenerator {
    pub fn new(access_key: String, secret_key: String) -> Self {
        ManageTokenGenerator {
            access_key,
            secret_key,
        }
    }

    fn sign(&self, sign_str: String) -> String {
        let hmac_digest = hmac_sha1::hmac_sha1(self.secret_key.as_bytes(), sign_str.as_bytes());
        let mut buf = String::new();
        BASE64_URL_SAFE.encode_string(hmac_digest, &mut buf);
        format!("{}:{}", self.access_key, buf)
    }

    ///
    /// 管理凭证[第一版算法](https://developer.qiniu.com/kodo/6671/historical-document-management-certificate)
    /// ```
    /// use qiniu_cdn_manager::token::ManageTokenGenerator;
    /// let generator = ManageTokenGenerator::new("MY_ACCESS_KEY".to_string(), "MY_SECRET_KEY".to_string());
    /// let url = "http://rs.qiniu.com/move/bmV3ZG9jczpmaW5kX21hbi50eHQ=/bmV3ZG9jczpmaW5kLm1hbi50eHQ=";
    /// let result = generator.generate_v1(url, None, None).unwrap();
    /// assert_eq!(result, "MY_ACCESS_KEY:FXsYh0wKHYPEsIAgdPD9OfjkeEM=");
    /// ```
    ///
    pub fn generate_v1(
        &self,
        url: &str,
        content_type: Option<&str>,
        body: Option<&[u8]>,
    ) -> Result<String, Error> {
        let url = Url::parse(url)?;
        let mut sign_str = url.path().to_string();
        if let Some(query) = url.query() {
            sign_str = format!("{sign_str}?{query}");
        }
        sign_str = format!("{sign_str}\n");
        if content_type.is_some_and(|x| x == "application/x-www-form-urlencoded")
            && body.is_some_and(|x| !x.is_empty())
        {
            sign_str = format!("{sign_str}{}", String::from_utf8_lossy(body.unwrap()));
        }
        Ok(self.sign(sign_str))
    }

    ///
    /// 管理凭证[第二版算法](https://developer.qiniu.com/kodo/1201/access-token)
    /// ```
    /// use qiniu_cdn_manager::token::ManageTokenGenerator;
    /// let generator = ManageTokenGenerator::new("MY_ACCESS_KEY".to_string(), "MY_SECRET_KEY".to_string());
    /// let url = "http://rs.qiniu.com/move/bmV3ZG9jczpmaW5kX21hbi50eHQ=/bmV3ZG9jczpmaW5kLm1hbi50eHQ=";
    /// let result = generator.generate_v2("POST", url, None, None, None).unwrap();
    /// assert_eq!(result, "MY_ACCESS_KEY:1uLvuZM6l6oCzZFqkJ6oI4oFMVQ=");
    /// ```
    pub fn generate_v2(
        &self,
        method: &str,
        url: &str,
        headers: Option<&reqwest::header::HeaderMap>,
        content_type: Option<&str>,
        body: Option<&[u8]>,
    ) -> Result<String, Error> {
        let url = Url::parse(url)?;
        let mut sign_str = format!("{} {}", method.to_uppercase(), url.path());
        if let Some(query) = url.query() {
            sign_str = format!("{sign_str}?{}", query);
        }
        sign_str = format!("{sign_str}\nHost: {}", url.host().unwrap());
        if content_type.is_some() {
            sign_str = format!("{sign_str}\nContent-Type: {}", content_type.unwrap());
        }
        let mut qiniu_headers: HashMap<&str, &str> = HashMap::new();
        if let Some(headers) = headers {
            for (k, v) in headers.iter() {
                let k = k.as_str();
                if let Some(stripped) = k.strip_prefix("X-Qiniu-") {
                    qiniu_headers.insert(stripped, v.to_str().unwrap());
                }
            }
        }
        if !qiniu_headers.is_empty() {
            let mut items: Vec<_> = qiniu_headers.into_iter().collect();
            items.sort_by(|a, b| a.0.cmp(b.0));
            for (k, v) in items.iter() {
                sign_str = format!("{sign_str}\n{}: {}", k, v);
            }
        }
        sign_str = format!("{sign_str}\n\n");
        if content_type.is_none() || content_type.is_some_and(|x| x != "application/octet-stream") {
            if let Some(body) = body {
                sign_str = format!("{sign_str}{}", String::from_utf8_lossy(body));
            }
        }
        Ok(self.sign(sign_str))
    }
}
