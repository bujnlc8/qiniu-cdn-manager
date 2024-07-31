//! 日志下载

#![allow(clippy::too_many_arguments)]

use anyhow::anyhow;
use chrono::{Duration, NaiveDate};
use colored::Colorize;
use flate2::read::GzDecoder;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufRead, Cursor, Write},
    path::{Path, PathBuf},
};

use std::{
    io::{copy, BufReader},
    sync::Arc,
};
use tokio::sync::{mpsc, Semaphore};

use serde::Deserialize;

use crate::{
    utils::{max_length, print_err, WaitBlinker},
    Client, NOT_FOUND_MSG,
};

#[derive(Debug, Deserialize, Clone)]
pub struct LogResponse {
    pub code: Option<i32>,
    pub error: String,
    pub data: Option<HashMap<String, Vec<LogData>>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LogData {
    pub name: String,
    pub size: i64,
    pub mtime: i64,
    pub url: String,
    pub md5: String,
}

impl Client {
    /// ### [日志下载](https://developer.qiniu.com/fusion/1226/download-the-log)
    pub async fn download(
        self,
        day: &str,
        download_dir: Option<&str>,
        limit: Option<i32>,
        unzip_keep: bool,
        unzip_not_keep: bool,
        domain: &str,
    ) -> Result<LogResponse, anyhow::Error> {
        let url = format!("https://{}{}", self.host, "/v2/tune/log/list");
        let mut data = HashMap::new();
        data.insert("day", day);
        data.insert("domains", domain);
        let domain = domain.to_string();
        let response: LogResponse = self
            .do_request::<LogResponse, HashMap<&str, &str>>(
                "POST",
                &url,
                None,
                Some("application/json"),
                Some(&data),
            )
            .await?;
        if let Some(download_dir) = download_dir {
            if let Some(log_data) = response.data.clone() {
                if let Some(log_list) = log_data.get(&domain) {
                    let mut limit = limit.unwrap_or(1000);
                    let mut tasks = Vec::new();
                    for log in log_list {
                        limit -= 1;
                        if limit < 0 {
                            break;
                        }
                        let this = self.clone();
                        let log = log.clone();
                        let download_dir = download_dir.to_owned().clone();
                        let domain = domain.clone();
                        let task = tokio::spawn(async move {
                            println!("开始下载 {} ...", &log.url.cyan());
                            this.download_with_url(
                                &log.url,
                                log.name.split('/').last().unwrap(),
                                &download_dir,
                                &domain,
                                unzip_keep,
                                unzip_not_keep,
                                &log.md5,
                            )
                            .await
                            .unwrap();
                            println!("下载完成 ✅ {}", &log.url.green());
                        });
                        tasks.push(task);
                    }
                    for task in tasks {
                        task.await?;
                    }
                    println!("{}", "全部下载完成 ✅".green());
                } else {
                    print_err(NOT_FOUND_MSG, false);
                }
            }
        }
        Ok(response)
    }

    async fn download_with_url(
        &self,
        url: &str,
        file_name: &str,
        download_dir: &str,
        domain: &str,
        unzip_keep: bool,
        unzip_not_keep: bool,
        md5: &str,
    ) -> Result<(), anyhow::Error> {
        let mut log_dir = PathBuf::from(download_dir);
        if self.config.download_log_domain_dir.unwrap_or(true) {
            log_dir = log_dir.join(PathBuf::from(domain));
        }
        if !log_dir.exists() {
            fs::create_dir_all(log_dir.clone())?;
        }
        let tmp_dir = PathBuf::from(format!("/tmp/qiniu/{}", md5));
        if !tmp_dir.exists() {
            fs::create_dir_all(tmp_dir.clone()).unwrap();
        };
        let tmp_file_path = tmp_dir.join(file_name.split('/').last().unwrap());
        if tmp_file_path.exists() {
            fs::copy(tmp_file_path, log_dir.join(file_name)).unwrap();
        } else {
            let response = reqwest::get(url).await?;
            let mut f = File::create(log_dir.join(file_name))?;
            let bytes = response.bytes().await?;
            f.write_all(&bytes)?;
            f.flush()?;
            // 保存临时文件
            let mut tmp_file = fs::File::create(tmp_file_path).unwrap();
            tmp_file.write_all(&bytes).unwrap();
            tmp_file.flush().unwrap();
        }
        if self.config.debug.unwrap_or(false) && (unzip_keep || unzip_not_keep) {
            println!(
                "[DEBUG] 开始解压缩 {}, unzip_keep: {}, unzip_not_keep: {}",
                file_name, unzip_keep, unzip_not_keep
            );
        }
        if unzip_keep {
            self.unzip(&log_dir.join(file_name), true)?;
        } else if unzip_not_keep {
            self.unzip(&log_dir.join(file_name), false)?;
        }
        Ok(())
    }

    fn unzip(&self, file_path: &Path, keep_file: bool) -> Result<(), anyhow::Error> {
        let input_file = File::open(file_path)?;
        let mut gz = GzDecoder::new(input_file);
        let output_file_path = file_path.to_str().unwrap().strip_suffix(".gz").unwrap();
        let mut output_file = File::create(Path::new(output_file_path))?;
        copy(&mut gz, &mut output_file)?;
        if !keep_file {
            fs::remove_file(file_path)?;
        }
        Ok(())
    }

    async fn multi_day_records(
        self,
        start_date: &str,
        end_date: &str,
        domain: String,
    ) -> Result<Vec<String>, anyhow::Error> {
        let start_date_dt = NaiveDate::parse_from_str(start_date, "%Y-%m-%d").unwrap();
        let end_date_dt = NaiveDate::parse_from_str(end_date, "%Y-%m-%d").unwrap();
        if start_date_dt > end_date_dt {
            return Err(anyhow!("开始日期不能大于结束日期！"));
        }
        let days = (end_date_dt - start_date_dt).num_days() + 1;
        if days > 30 {
            return Err(anyhow!("间隔不能大于30天！"));
        }
        let (tx, mut rx) = mpsc::channel(days as usize);
        for i in 0..days {
            let dt = start_date_dt + Duration::days(i);
            let tx1 = tx.clone();
            let this = self.clone();
            let domain = domain.clone();
            tokio::spawn(async move {
                if this.config.debug.unwrap_or(false) {
                    println!("[DEBUG] Log dt: {}", dt);
                }
                let response = this
                    .download(
                        dt.format("%Y-%m-%d").to_string().as_str(),
                        None,
                        None,
                        false,
                        false,
                        &domain,
                    )
                    .await
                    .unwrap();
                if let Some(data) = response.data {
                    if let Some(log) = data.get(domain.as_str()) {
                        tx1.send(log.clone()).await.unwrap();
                    }
                }
                drop(tx1);
            });
        }
        drop(tx);
        let mut log_datas: Vec<LogData> = vec![];
        while let Some(log_data) = rx.recv().await {
            log_datas.extend(log_data);
        }
        let (tx, mut rx) = mpsc::channel(log_datas.len() + 1);
        // 限制并发的数量为25，太大服务容易挂
        let semaphore = Arc::new(Semaphore::new(25));
        for log_data in log_datas {
            let tx1 = tx.clone();
            let this = self.clone();
            let semaphore = semaphore.clone();
            tokio::spawn(async move {
                let semaphore = semaphore.clone();
                let name = log_data.name;
                let md5 = log_data.md5;
                let tmp_dir = PathBuf::from(format!("/tmp/qiniu/{}", md5));
                if !tmp_dir.exists() {
                    fs::create_dir_all(tmp_dir.clone()).unwrap();
                };
                let tmp_file_path = tmp_dir.join(name.split('/').last().unwrap());
                let bytes = if tmp_file_path.exists() {
                    fs::read(tmp_file_path).unwrap()
                } else {
                    let url = log_data.url;
                    if this.config.debug.unwrap_or(false) {
                        println!("[DEBUG] Log url: {}", url);
                    }
                    let _permit = semaphore.acquire().await.unwrap();
                    let response = reqwest::get(url).await.unwrap();
                    drop(_permit);
                    let bytes = response.bytes().await.unwrap();
                    let mut tmp_file = fs::File::create(tmp_file_path).unwrap();
                    tmp_file.write_all(&bytes).unwrap();
                    tmp_file.flush().unwrap();
                    bytes.into()
                };
                let gz = GzDecoder::new(Cursor::new(bytes));
                let output = BufReader::new(gz);
                let mut records = Vec::new();
                for record in output.lines().map_while(Result::ok) {
                    records.push(record);
                }
                tx1.send(records).await.unwrap();
                drop(tx1);
            });
        }
        drop(tx);
        let mut records = Vec::new();
        while let Some(data) = rx.recv().await {
            records.extend(data);
        }
        Ok(records)
    }

    pub async fn ip_url(
        self,
        ip: &str,
        start_date: &str,
        end_date: &str,
        limit: Option<i32>,
        wait_blink: Option<WaitBlinker>,
        domain: &str,
    ) -> Result<(), anyhow::Error> {
        let log_records = match self
            .clone()
            .multi_day_records(start_date, end_date, domain.to_string())
            .await
        {
            Ok(k) => k,
            Err(e) => return Err(e),
        };
        let mut url_count_map = HashMap::new();
        for record in log_records {
            if record.trim().starts_with(ip) {
                let u = self.parse_url(record.trim());
                if u.is_empty() {
                    continue;
                }
                match url_count_map.get(&u) {
                    Some(k) => {
                        url_count_map.insert(u, k + 1);
                    }
                    None => {
                        url_count_map.insert(u, 1i32);
                    }
                }
            }
        }
        if let Some(blinker) = wait_blink {
            blinker.sender.send(true).unwrap();
            blinker.handle.await?;
        }
        if url_count_map.is_empty() {
            println!("{}", "没有找到该IP的请求日志".red());
            return Ok(());
        }
        let mut hash_vec: Vec<(&String, &i32)> = url_count_map.iter().collect();
        hash_vec.sort_by(|a, b| b.1.cmp(a.1));
        println!(
            "域名 {} IP {} {}～{} URL请求次数如下: ",
            domain.bold().yellow(),
            ip.bold().yellow(),
            start_date.bold(),
            end_date.bold(),
        );
        let limit = limit.unwrap_or(10000);
        let width = max_length(&url_count_map.keys(), limit);
        println!(
            "{:^width$} {:^10}",
            "URL".bold().green(),
            "Count".bold().green(),
            width = width,
        );
        for (url, c) in hash_vec.iter().take(limit as usize) {
            println!("{:<width$} {:^10}", url, c);
        }
        println!(
            "{:^width$} {:^10}",
            "Total".bold().green(),
            url_count_map.values().sum::<i32>(),
            width = width,
        );
        Ok(())
    }

    fn parse_url(&self, record: &str) -> String {
        if let Some(part) = record.split('"').skip(1).take(1).next() {
            let s = part.split(' ').skip(1).take(1).last().unwrap();
            return s.to_string();
        }
        "".to_string()
    }

    pub async fn filter_log(
        self,
        filter_string: Vec<String>,
        start_date: &str,
        end_date: &str,
        output_file: bool,
        wait_blink: Option<WaitBlinker>,
        domain: &str,
    ) -> Result<(), anyhow::Error> {
        let log_records = match self
            .clone()
            .multi_day_records(start_date, end_date, domain.to_string())
            .await
        {
            Ok(k) => k,
            Err(e) => return Err(e),
        };
        if let Some(wait_blink) = wait_blink {
            wait_blink.sender.send(true).unwrap();
            wait_blink.handle.await?;
        }
        if !output_file {
            let mut total = 0;
            log_records
                .iter()
                .filter(|x| {
                    for f in filter_string.clone() {
                        // 以!!开头表示不包含
                        if f.starts_with("!!") {
                            let f: String = f.chars().skip(2).collect();
                            if x.contains(&f) {
                                return false;
                            }
                        } else if !x.contains(&f) {
                            return false;
                        }
                    }
                    true
                })
                .for_each(|x| {
                    println!("{}", x);
                    total += 1;
                });
            println!("{}{}", "Total: ".cyan().bold(), total);
        } else {
            let file_name = format!(
                "{}.{}-{}-{}.log",
                domain,
                filter_string.first().unwrap(),
                start_date,
                end_date
            );
            let mut f = fs::File::create(PathBuf::from(file_name.clone()))?;
            let filter_logs: Vec<String> = log_records
                .iter()
                .filter(|x| {
                    for f in filter_string.clone() {
                        // 以!!开头表示不包含
                        if f.starts_with("!!") {
                            let f: String = f.chars().skip(2).collect();
                            if x.contains(&f) {
                                return false;
                            }
                        } else if !x.contains(&f) {
                            return false;
                        }
                    }
                    true
                })
                .map(|x| (*x).clone())
                .collect();
            f.write_all(filter_logs.join("\n").as_bytes())?;
            f.flush()?;
            println!("{}{}", "Total: ".cyan().bold(), filter_logs.len());
            println!(
                "包含`{}`的日志已导出: {}",
                filter_string.join(" ").to_string().red(),
                file_name.green().bold()
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
    async fn download_log_test() {
        let config = Config::parse(Some(PathBuf::from("./qiniu-cdn.toml")));
        let log = Client::new(&config, crate::SubFunctionEnum::Log);
        log.download(
            "2024-07-16",
            Some("./logs"),
            Some(1),
            false,
            false,
            &config.cdn.domain,
        )
        .await
        .unwrap();
    }
}
