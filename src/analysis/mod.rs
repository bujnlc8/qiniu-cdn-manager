//! 日志分析

pub mod count;
pub mod hitmiss;
pub mod isp;
pub mod status;
pub mod top;

use std::fmt::Display;

use serde::Serialize;

#[derive(Debug, Serialize, Clone, Copy, PartialEq, PartialOrd)]
pub enum Freq {
    FiveMin,
    OneHour,
    OneDay,
}

impl From<String> for Freq {
    fn from(value: String) -> Self {
        if value == "1day" {
            return Freq::OneDay;
        } else if value == "1hour" {
            return Freq::OneHour;
        } else if value == "5min" {
            return Freq::FiveMin;
        }
        panic!("无效的value")
    }
}

impl Display for Freq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Freq::FiveMin => f.write_str("5min"),
            Freq::OneDay => f.write_str("1day"),
            Freq::OneHour => f.write_str("1hour"),
        }
    }
}
