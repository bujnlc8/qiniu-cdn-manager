//! 配置文件

#![allow(deprecated)]

use std::{env, fs, path::PathBuf, process::exit};

use colored::Colorize;
use serde::Deserialize;

use crate::utils::print_err;

/// cdn config
#[derive(Deserialize, Debug, Clone)]
pub struct CDNConfig {
    pub access_key: String,
    pub secret_key: String,
    pub domain: String,
}

/// config
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub debug: Option<bool>,
    pub download_log_domain_dir: Option<bool>,
    pub cdn: CDNConfig,
    pub monitor: Monitor,
    pub blackip: BlackIP,
    pub five_minute_traffic: Option<i64>,
    pub five_minute_count: Option<i64>,
}

/// monitor config
#[derive(Deserialize, Debug, Clone)]
pub struct Monitor {
    pub qy_robot: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BlackIP {
    pub policy: Option<String>,
    pub rewrite: Option<bool>,
}

impl Config {
    /// parse config from path
    pub fn parse(config_path: Option<PathBuf>) -> Self {
        let config_path = match config_path {
            Some(k) => {
                if !k.exists() {
                    print_err(
                        format!(
                            "[ERR] {}{}",
                            "配置文件不存在: ".red(),
                            k.display().to_string().green()
                        )
                        .as_str(),
                        true,
                    )
                }
                k
            }
            None => {
                let current = PathBuf::from("./qiniu-cdn.toml");
                if current.exists() {
                    current
                } else {
                    let p = env::home_dir()
                        .unwrap()
                        .join(PathBuf::from(".config/qiniu-cdn.toml"));
                    if p.exists() {
                        p
                    } else {
                        print_err("配置文件不存在！", true);
                        exit(1);
                    }
                }
            }
        };
        let config_str = fs::read_to_string(config_path).unwrap();
        toml::from_str(&config_str).unwrap()
    }
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use super::*;
    #[test]
    fn test_parse() {
        let config_path = PathBuf::from("./qiniu-cdn.toml.example");
        let config = Config::parse(Some(config_path));
        assert_eq!(config.cdn.access_key, "abcd");
        assert_eq!(config.cdn.secret_key, "1234");
        assert_eq!(config.cdn.domain, "static.example.com");
        assert!(config.debug.unwrap())
    }
}
