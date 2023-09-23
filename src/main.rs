mod calendar;
mod config;
mod oauth;

use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, Duration, FixedOffset};
use oauth::get_access_token;

use crate::{calendar::get_calenders, config::read_config};

#[tokio::main]
async fn main() {
    let config = match read_config().await {
        Ok(c) => c,
        Err(e) => {
            println!("config.yamlが読み込めませんでした。{:?}", e);
            std::process::exit(1);
        }
    };
    println!("{:?}", config);

    let result = get_access_token().await;

    let token = result.unwrap();
    println!("{:?}", token.access_token);

    let events = get_calenders(
        &token.access_token,
        &config.calendar_id,
        &config.start,
        &config.end,
    )
    .await
    .unwrap();
    let mut total_duration = Duration::zero();

    let mut duration_map: HashMap<String, Duration> = HashMap::new();
    for event in events {
        if let (Some(summary), Some(start), Some(end)) =
            (event.summary, event.start.date_time, event.end.date_time)
        {
            let start_time = DateTime::<FixedOffset>::from_str(&start).unwrap();
            let end_time = DateTime::<FixedOffset>::from_str(&end).unwrap();
            let duration = end_time - start_time;
            total_duration = total_duration + duration;
            let entry = duration_map.entry(summary).or_insert(Duration::seconds(0));
            *entry = *entry + duration;
        }
    }
    for (summary, duration) in &duration_map {
        println!("{}: {:?} min", summary, duration.num_minutes());
    }
    println!("合計時間: {:?} min", total_duration.num_minutes());
}
