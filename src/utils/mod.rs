//! utils
pub mod qy_robot;
pub mod region_isp;
pub mod token;
use std::process::exit;
use std::{
    io::{self, Write},
    time::Duration,
};

use colored::Colorize;
use dialoguer::{console::Style, theme::ColorfulTheme, Confirm};
use tokio::{
    sync::oneshot::{self, Sender},
    task::JoinHandle,
    time::sleep,
};

use crate::{config::Config, Client};

pub fn print_err<T: Colorize>(msg: T, exit_process: bool) {
    eprintln!("[ERR] {}", msg.red());
    std::io::stderr().flush().unwrap_or(());
    if exit_process {
        exit(1)
    }
}

pub fn prompt<T: Into<String>>(msg: T, theme: Option<ColorfulTheme>) -> bool {
    let theme = theme.unwrap_or_else(|| ColorfulTheme {
        prompt_style: Style::new().blue().bold(),
        ..Default::default()
    });
    Confirm::with_theme(&theme)
        .with_prompt(msg)
        .default(false)
        .show_default(true)
        .wait_for_newline(false)
        .interact()
        .unwrap_or(false)
}

pub fn clear_current_line() {
    // 使用 ANSI 转义序列清除行并将光标移到行首
    print!("\r\x1B[2K");
    io::stdout().flush().unwrap();
}

pub fn clear_previous_line() {
    // 使用 ANSI 转义序列移动光标到上一行并清除该行
    print!("\x1B[1A\x1B[2K");
    io::stdout().flush().unwrap();
}

pub fn max_length<T: IntoIterator<Item: Into<String>> + Clone>(data: &T, limit: i32) -> usize {
    let mut length = 0;
    for item in data.clone().into_iter().take(limit as usize) {
        let item: String = item.into();
        if length < item.len() {
            length = item.len();
        }
    }
    length
}

#[derive(Debug)]
pub struct WaitBlinker {
    pub sender: Sender<bool>,
    pub handle: JoinHandle<()>,
}

pub fn wait_blink(msg: String, blink_char_num: usize) -> WaitBlinker {
    let (tx, mut rx) = oneshot::channel::<bool>();
    let handle = tokio::spawn(async move {
        loop {
            print!("{}", format!("\r{}", msg).green());
            io::stdout().flush().unwrap();
            sleep(Duration::from_millis(120)).await;
            print!(
                "{}",
                format!(
                    "\r{}{}",
                    msg.chars()
                        .take(msg.chars().count() - blink_char_num)
                        .collect::<String>(),
                    " ".repeat(blink_char_num),
                )
                .green()
            );
            io::stdout().flush().unwrap();
            sleep(Duration::from_millis(50)).await;
            if rx.try_recv().is_ok() {
                clear_current_line();
                break;
            }
        }
    });
    WaitBlinker { sender: tx, handle }
}

pub async fn get_domains(
    config: &Config,
    exclude_domains: &Option<String>,
    domains: &Option<String>,
) -> Result<Vec<String>, anyhow::Error> {
    let tmp = exclude_domains.clone().unwrap_or("".to_string());
    let exclude_domains: Vec<_> = tmp.split(',').collect();
    let mut res = vec![];
    if let Some(domains) = domains {
        for domain in domains.split(',') {
            if !exclude_domains.contains(&domain) {
                res.push(domain.to_string());
            }
        }
    } else {
        let client = Client::new(config, crate::SubFunctionEnum::Domain);
        let response = client.domain_list().await?;
        response.domains.iter().for_each(|x| {
            if !exclude_domains.contains(&x.name.as_str()) {
                res.push(x.name.clone());
            }
        });
    }
    Ok(res)
}
