use std::fs;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub start: String,
    pub end: String,
    pub calendar_id: String,
}

#[derive(Debug)]
pub struct ConfigError {
    pub message: String,
}

pub async fn read_config() -> Result<Config, ConfigError> {
    let config_str = fs::read_to_string("config.yaml").map_err(|op| ConfigError {
        message: op.to_string(),
    })?;
    let config: Config = serde_yaml::from_str(&config_str).map_err(|op| ConfigError {
        message: op.to_string(),
    })?;
    Ok(config)
}
