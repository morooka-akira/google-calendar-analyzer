use std::{fmt, fs, str::FromStr};

use chrono::NaiveTime;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ConfigFile {
    start_date: String,
    end_date: String,
    calendar_id: String,
    day_of_weeks: String,
    start_time: String,
    end_time: String,
}

#[derive(Debug)]
pub struct Config {
    pub start_date: String,
    pub end_date: String,
    pub calendar_id: String,
    pub day_of_weeks: Vec<u8>,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Config:")?;
        writeln!(f, "  Start Date: {}", self.start_date)?;
        writeln!(f, "  End Date: {}", self.end_date)?;
        writeln!(f, "  Calendar ID: {}", self.calendar_id)?;
        writeln!(
            f,
            "  Days of Week: {}",
            self.day_of_weeks
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )?;
        writeln!(f, "  Start Time: {}", self.start_time)?;
        writeln!(f, "  End Time: {}", self.end_time)
    }
}

#[derive(Debug)]
pub struct ConfigError {
    pub message: String,
}

pub async fn read_config() -> Result<Config, ConfigError> {
    let config_str = fs::read_to_string("config.yaml").map_err(|op| ConfigError {
        message: op.to_string(),
    })?;
    let file: ConfigFile = serde_yaml::from_str(&config_str).map_err(|err| ConfigError {
        message: err.to_string(),
    })?;
    Ok(Config {
        start_date: file.start_date,
        end_date: file.end_date,
        calendar_id: file.calendar_id,
        day_of_weeks: day_of_weeks(&file.day_of_weeks),
        start_time: file.start_time.to_naive_time(),
        end_time: file.end_time.to_naive_time(),
    })
}

fn day_of_weeks(day_of_week_str: &str) -> Vec<u8> {
    day_of_week_str
        .split(',')
        .filter_map(|m| m.parse().ok())
        .collect()
}

trait ToNaiveTime {
    fn to_naive_time(&self) -> NaiveTime;
}

impl ToNaiveTime for String {
    fn to_naive_time(&self) -> NaiveTime {
        NaiveTime::from_str(self).expect("時間はHH:MM形式で入力してください。")
    }
}
