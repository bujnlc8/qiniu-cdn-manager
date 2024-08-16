//! 域名相关

use std::{
    collections::{HashMap, HashSet},
    fs,
    io::Write,
};

use anyhow::anyhow;
use chrono::{DateTime, Duration, NaiveDate};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::{
    analysis::top::FilterType,
    utils::{print_err, prompt, qy_robot::QyRobot},
    Client,
};
use colored::Colorize;

#[derive(Debug, Deserialize, Clone)]
pub struct Response {
    pub code: Option<i32>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct IpACLParam<'a> {
    #[serde(rename = "ipACLType")]
    pub ip_acltype: &'a str,
    #[serde(rename = "ipACLValues")]
    pub ip_aclvalues: Vec<&'a str>,
}

pub enum IpACLType {
    White,
    Black,
    Blank,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IpACL {
    #[serde(rename = "ipACLType")]
    pub ip_acltype: String,
    #[serde(rename = "ipACLValues")]
    pub ip_aclvalues: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Referer {
    #[serde(rename = "refererType")]
    pub referer_type: String,
    #[serde(rename = "refererValues")]
    pub referer_values: Vec<String>,
    #[serde(rename = "nullReferer")]
    pub null_referer: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Https {
    #[serde(rename = "certId")]
    pub cert_id: String,
    #[serde(rename = "forceHttps")]
    pub force_https: bool,
    #[serde(rename = "http2Enable")]
    pub http2_enable: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DomainInfoResponse {
    pub code: Option<i32>,
    pub error: Option<String>,
    pub name: Option<String>,
    pub cname: Option<String>,
    #[serde(rename = "ipACL")]
    pub ip_acl: Option<IpACL>,
    pub referer: Option<Referer>,
    #[serde(rename = "createAt")]
    pub create_at: Option<String>,
    #[serde(rename = "modifyAt")]
    pub modify_at: Option<String>,
    #[serde(rename = "registerNo")]
    pub register_no: Option<String>,
    pub https: Option<Https>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CertResponse {
    pub code: i32,
    pub error: String,
    pub cert: CertData,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CertData {
    pub name: String,
    pub common_name: String,
    pub dnsnames: Vec<String>,
    pub not_before: i64,
    pub not_after: i64,
    pub pri: String,
    pub ca: String,
    pub create_time: i64,
    pub enable: bool,
}

#[derive(Debug, Deserialize)]
pub struct DomainListResponse {
    pub marker: String,
    pub domains: Vec<DomainListInner>,
}

#[derive(Debug, Deserialize)]
pub struct DomainListInner {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub cname: Option<String>,
    pub protocol: Option<String>,
    #[serde(rename = "operationType")]
    pub operation_type: Option<String>,
    #[serde(rename = "operatingState")]
    pub operating_state: Option<String>,
    #[serde(rename = "createAt")]
    pub create_at: String,
    #[serde(rename = "modifyAt")]
    pub modify_at: String,
}

impl Client {
    /// ### [修改ip黑白名单](https://developer.qiniu.com/fusion/4246/the-domain-name#16)
    pub async fn ip_acl(
        &self,
        ips: Vec<&str>,
        ip_acltype: IpACLType,
        domain: &str,
    ) -> Result<Response, anyhow::Error> {
        let url = format!("https://{}{}{domain}/ipacl", self.host, "/domain/");
        let ip_acltype = match ip_acltype {
            IpACLType::White => "white",
            IpACLType::Black => "black",
            IpACLType::Blank => "",
        };
        let data = IpACLParam {
            ip_acltype,
            ip_aclvalues: ips,
        };
        let response = self
            .do_request::<Response, IpACLParam>(
                "PUT",
                &url,
                None,
                Some("application/json"),
                Some(&data),
            )
            .await?;
        Ok(response)
    }

    pub async fn domain_info(&self, domain: &str) -> Result<DomainInfoResponse, anyhow::Error> {
        let url = format!("https://{}/domain/{domain}", self.host);
        let response = self
            .do_request::<DomainInfoResponse, IpACLParam>(
                "GET",
                &url,
                None,
                Some("application/json"),
                None,
            )
            .await?;
        Ok(response)
    }

    pub async fn cert(&self, cert_id: &str) -> Result<CertResponse, anyhow::Error> {
        let url = format!("https://{}/sslcert/{}", self.host, cert_id);
        let mut header = HeaderMap::new();
        header.insert(
            "Content-Type",
            HeaderValue::from_str("application/x-www-form-urlencoded").unwrap(),
        );
        let response = self
            .do_request::<CertResponse, IpACLParam>(
                "GET",
                &url,
                Some(&header),
                Some("application/x-www-form-urlencoded"),
                None,
            )
            .await?;
        Ok(response)
    }
    pub async fn print_domain_info(
        &self,
        response: &DomainInfoResponse,
        download_ssl_cert: bool,
        domain: &str,
    ) {
        response.error.is_some().then(|| {
            print_err(response.error.clone().unwrap().as_str(), true);
        });
        println!("域名 {} 信息如下: ", domain.bold().yellow());
        println!(
            "{}{}",
            "CName: ".green().bold(),
            response.cname.clone().unwrap()
        );
        println!(
            "{}{}",
            "创建时间: ".green().bold(),
            response.create_at.clone().unwrap()
        );
        println!(
            "{}{}",
            "修改时间: ".green().bold(),
            response.modify_at.clone().unwrap()
        );
        println!(
            "{}{}",
            "备案号: ".green().bold(),
            response.register_no.clone().unwrap_or("无".to_string())
        );
        let acl = response.ip_acl.clone().unwrap();
        if acl.ip_acltype.is_empty() {
            println!("{}关闭", "IP黑白名单: ".green().bold());
        } else {
            println!("{}开启", "IP黑白名单: ".green().bold());
            if acl.ip_acltype == "black" {
                println!("{:>10}黑名单", "模式: ".bold());
            } else {
                println!("{:>10}白名单", "模式: ".bold());
            }
            println!("{:>10}", "列表: ".bold());
            for k in acl.ip_aclvalues {
                println!("{:>24}", k);
            }
        }
        let referer = response.referer.clone().unwrap();
        if referer.referer_type.is_empty() {
            println!("{}关闭", "Referer防盗链: ".green().bold());
        } else {
            println!("{}开启", "Referer防盗链: ".green().bold());
            if referer.referer_type == "black" {
                println!("{:>10}黑名单", "模式: ".bold());
            } else {
                println!("{:>10}白名单", "模式: ".bold());
            }
            println!("{:>10}{}", "空Referer: ".bold(), referer.null_referer);
            println!("{:>10}", "列表: ".bold());
            for k in referer.referer_values {
                println!("{:>24}", k);
            }
        }
        let https = response.https.clone();
        if let Some(https) = https {
            println!("{}开启", "开启HTTPS: ".green().bold());
            if https.force_https {
                println!("{}开启", "强制HTTPS: ".green().bold());
            } else {
                println!("{}关闭", "强制HTTPS: ".green().bold());
            }
            if https.http2_enable {
                println!("{}开启", "HTTP/2访问: ".green().bold());
            } else {
                println!("{}关闭", "HTTP/2访问: ".green().bold());
            }
            // 查询证书
            let cert = self.cert(&https.cert_id).await.unwrap();
            let start_time = DateTime::from_timestamp(cert.cert.not_before, 0).unwrap();
            println!(
                "{}{}",
                "SSL证书开始时间: ".green().bold(),
                start_time.format("%Y-%m-%d %H:%M:%S")
            );
            let valid_time = DateTime::from_timestamp(cert.cert.not_after, 0).unwrap();
            println!(
                "{}{}",
                "SSL证书到期时间: ".green().bold(),
                valid_time.format("%Y-%m-%d %H:%M:%S")
            );
            if download_ssl_cert {
                let key_file = format!("{}.key", domain);
                let ca_file = format!("{}_ca.cert", domain);
                let mut key = fs::File::create(&key_file).unwrap();
                key.write_all(cert.cert.pri.as_bytes()).unwrap();
                key.flush().unwrap();
                let mut ca = fs::File::create(&ca_file).unwrap();
                ca.write_all(cert.cert.ca.as_bytes()).unwrap();
                ca.flush().unwrap();
                println!(
                    "{}{},{}",
                    "证书已下载到当前目录: ".yellow(),
                    key_file,
                    ca_file
                );
            }
        } else {
            println!("{}关闭", "开启HTTPS: ".green().bold());
        }
    }

    pub async fn set_ip_acl(
        &self,
        black: bool,
        white: bool,
        close: bool,
        ips: &str,
        rewrite: bool,
        domain: &str,
    ) -> Result<usize, anyhow::Error> {
        let mut check_num = 0;
        let mut ip_acltype = IpACLType::Blank;
        if black {
            check_num += 1;
            ip_acltype = IpACLType::Black;
        }
        if white {
            check_num += 1;
            ip_acltype = IpACLType::White;
        }
        if close {
            check_num += 1;
            ip_acltype = IpACLType::Blank;
        }
        // 参数互斥
        if check_num != 1 {
            print_err("参数错误", true);
        }
        if (white || black) && ips.is_empty() {
            print_err("ips参数错误", true);
        }
        // 关闭黑白名单
        if close {
            self.ip_acl(vec![], IpACLType::Blank, domain).await?;
            println!("{}", "关闭成功 ✅".green());
            return Ok(0);
        }
        let ips: Vec<&str> = ips.split(',').collect();
        if ips.is_empty() {
            print_err("ip列表为空！", true);
        }
        // 不是重写模式，先查询，再追加
        if !rewrite {
            let domain_info = self.domain_info(domain).await?;
            let origin_ip_acl = domain_info.ip_acl.clone().unwrap();
            let (remove_ips, mut ips): (Vec<&str>, Vec<&str>) =
                ips.iter().partition(|&s| s.starts_with('d'));
            let mut should_merge = false;
            if origin_ip_acl.ip_acltype == "white" && white {
                should_merge = true;
            }
            if origin_ip_acl.ip_acltype == "black" && black {
                should_merge = true;
            }
            if should_merge {
                for ip in origin_ip_acl.ip_aclvalues.iter() {
                    if ips.contains(&(*ip).as_str()) {
                        continue;
                    }
                    if remove_ips.contains(&format!("d{}", *ip).as_str()) {
                        continue;
                    }
                    ips.push(ip.as_str());
                }
                // 检查ip名单是否相同
                let mut should_invoke_api = false;
                for ip in ips.iter() {
                    if !origin_ip_acl.ip_aclvalues.contains(&(*ip).to_string()) {
                        should_invoke_api = true;
                        break;
                    }
                }
                if !should_invoke_api && !ips.is_empty() {
                    println!("{}", "与线上配置相同，跳过设置！".yellow());
                    return Ok(0);
                }
            } else if !remove_ips.is_empty() {
                return Err(anyhow!("当前IP模式和线上不一致，无法移除IP！"));
            }
            if ips.is_empty() {
                println!("[WARN] {}", "ip列表为空，将IP黑/白名单关闭".yellow());
                ip_acltype = IpACLType::Blank;
            }
            let response = self.ip_acl(ips.clone(), ip_acltype, domain).await?;
            if response.code.is_some() && response.code.unwrap() != 200 {
                return Err(anyhow!("{}", response.error.unwrap()));
            }
            println!("{}", "操作成功 ✅".green());
            return Ok(ips.len());
        } else {
            let response = self.ip_acl(ips.clone(), ip_acltype, domain).await?;
            if response.code.is_some() && response.code.unwrap() != 200 {
                return Err(anyhow!("{}", response.error.unwrap()));
            }
            println!("{}", "操作成功 ✅".green());
            return Ok(ips.len());
        }
    }

    pub async fn diagnose_ip(
        &self,
        domain: &str,
        day: &str,
    ) -> Result<HashSet<String>, anyhow::Error> {
        let policy = self.config.blackip.policy.clone();
        if policy.is_none() || policy.clone().unwrap().is_empty() {
            print_err("policy字段未配置！", true);
        }
        let policy = policy.unwrap();
        let mut policies = vec![];
        for s in policy.split("||") {
            for x in s.split("&&") {
                policies.push(x);
            }
        }
        if policies.len() > 2 {
            print_err("policy字段配置错误", true);
        }
        let mut mode = "and";
        if policy.contains("||") {
            mode = "or";
        }
        let mut ips: Vec<HashSet<String>> = vec![];
        let analysis_client = Client::new(&self.config, crate::SubFunctionEnum::AnalysisTop);
        for p in policies {
            let ps: Vec<&str> = p.split(":").collect();
            if ps.len() != 3 {
                print_err("policy字段配置错误", true);
            }
            if *ps.first().unwrap() != "T" && *ps.first().unwrap() != "C" {
                print_err("policy字段配置错误", true);
            }
            let mut filter_type = FilterType::Traffic;
            if *ps.first().unwrap() == "C" {
                filter_type = FilterType::ReqCount;
            }
            let days: i64 = ps.get(1).unwrap().parse::<i64>().unwrap();
            let num: i64 = ps.get(2).unwrap().parse::<i64>().unwrap();
            let mut start_dt = NaiveDate::parse_from_str(day, "%Y-%m-%d").unwrap();
            if days > 1 {
                start_dt -= Duration::days(days - 1);
            }
            if days < 1 {
                print_err("policy字段配置错误", true);
            }
            let start_date = start_dt.format("%Y-%m-%d").to_string();
            let response = analysis_client
                .top_ip(
                    "global",
                    &start_date,
                    day,
                    filter_type,
                    vec![domain.to_string()],
                )
                .await?;
            if response.code != 200 {
                print_err(
                    format!(
                        "接口响应错误, code: {}, message: {}",
                        response.code, response.error
                    )
                    .as_str(),
                    true,
                );
            }
            if response.data.is_none() {
                if self.config.debug.unwrap_or(false) {
                    print_err("未查询到流量数据", false);
                }
                return Ok(HashSet::new());
            }
            if self.config.debug.unwrap_or(false) {
                println!(
                    "start_date: {} end_date: {} num: {} days: {}, mode: {:#?}",
                    start_date, day, num, days, filter_type
                );
            }
            let data = response.data.unwrap();
            let mut ip_result = HashSet::new();
            let ip_arr = data.ips.unwrap();
            if filter_type == FilterType::ReqCount {
                for (i, c) in data.count.unwrap().iter().enumerate() {
                    if *c >= num {
                        let s = ip_arr.get(i).unwrap();
                        ip_result.insert(s.clone());
                    }
                }
            } else {
                for (i, c) in data.traffic.unwrap().iter().enumerate() {
                    if *c >= num * 1024 * 1024 {
                        let s = ip_arr.get(i).unwrap();
                        ip_result.insert(s.clone());
                    }
                }
            }
            ips.push(ip_result);
        }
        if self.config.debug.unwrap_or(false) {
            println!("ips: {:#?}", ips);
        }
        let mut result = HashSet::new();
        if mode == "or" {
            for ip in ips {
                for ip_ in ip {
                    result.insert(ip_);
                }
            }
        } else {
            let mut ip_map = HashMap::new();
            for ip in ips.clone() {
                for ip_ in ip {
                    let exist = ip_map.get(&ip_).unwrap_or(&0);
                    ip_map.insert(ip_, exist + 1);
                }
            }
            let count = ips.len();
            for (ip, v) in ip_map {
                if v == count {
                    result.insert(ip);
                }
            }
        }
        Ok(result)
    }

    pub async fn domain_list(&self) -> Result<DomainListResponse, anyhow::Error> {
        let url = format!("https://{}{}?types=normal&limit=1000", self.host, "/domain");
        let response = self
            .do_request::<DomainListResponse, Option<()>>(
                "GET",
                &url,
                None,
                Some("application/x-www-form-urlencoded"),
                None,
            )
            .await?;
        Ok(response)
    }

    pub async fn process_diagnostic_ips(
        &self,
        ips: HashSet<String>,
        apply_black_ip: bool,
        no_prompt: bool,
        no_qy_notify: bool,
        domain: &str,
    ) -> Result<(), anyhow::Error> {
        // let ips = vec!["27.115.124.49", "14.153.217.67"];
        println!("域名 {} IP诊断结果: ", domain.yellow().bold());
        if ips.is_empty() {
            println!("{}", "未诊断出符合条件的IP".red());
            return Ok(());
        }
        println!("{}", "诊断出符合条件的IP: ".green());
        let mut ipss = vec![];
        for ip in ips {
            println!("{}", ip);
            ipss.push(ip);
        }
        // 不设置黑名单
        if !apply_black_ip {
            return Ok(());
        }
        if !no_prompt && !prompt("将上面诊断出的IP配置成黑名单?", None) {
            return Ok(());
        }
        let mut mode = "覆盖";
        if !self.config.blackip.rewrite.unwrap_or(true) {
            mode = "追加";
        }
        let msg = format!("## 🔔七牛云CDN IP黑/白名单修改\n\n`{}`采用`{}`模式添加了以下IP到黑名单:\n\n- {}\n\n> `d`开头表示移除\n\n🚀🚀🚀",
                        domain,
                        mode,
                        ipss.join("\n\n- ")
        );
        if self.config.blackip.clone().rewrite.unwrap_or(true) {
            if !no_prompt && !prompt("将采用覆盖模式，将覆盖线上配置?", None) {
                return Ok(());
            }
            let ip_num = self
                .set_ip_acl(true, false, false, ipss.join(",").as_str(), true, domain)
                .await?;
            if ip_num > 0 && !no_qy_notify && self.config.monitor.qy_robot.is_some() {
                QyRobot::new(self.config.monitor.clone().qy_robot.unwrap())
                    .send_message(&msg)
                    .await?;
            }
        } else {
            if !no_prompt && !prompt("将采用追加模式，不会覆盖线上配置?", None) {
                return Ok(());
            }
            let ip_num = self
                .set_ip_acl(true, false, false, ipss.join(",").as_str(), false, domain)
                .await?;
            if ip_num > 0 && !no_qy_notify && self.config.monitor.qy_robot.is_some() {
                QyRobot::new(self.config.monitor.clone().qy_robot.unwrap())
                    .send_message(&msg)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn all_domain_diagnostic(
        &mut self,
        apply_black_ip: bool,
        no_prompt: bool,
        no_qy_notify: bool,
        domains: Vec<String>,
        day: &str,
    ) -> Result<(), anyhow::Error> {
        for d in domains {
            let ips = self.diagnose_ip(&d, day).await?;
            self.process_diagnostic_ips(ips, apply_black_ip, no_prompt, no_qy_notify, &d)
                .await?;
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
    async fn ip_acl_test() {
        let config = Config::parse(Some(PathBuf::from("./qiniu-cdn.toml")));
        let domain = Client::new(&config, crate::SubFunctionEnum::Domain);
        domain
            .ip_acl(vec![], IpACLType::Blank, &config.cdn.domain)
            .await
            .unwrap();
    }
}
