use anyhow::Result;
use std::str::FromStr;

use chrono::{DateTime, Datelike, FixedOffset, NaiveTime};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client,
};
use serde::Deserialize;
use urlencoding::encode;

use crate::config;

#[derive(Deserialize, Debug)]
struct Events {
    items: Vec<EventItem>,
}

#[derive(Deserialize, Debug)]
struct EventItem {
    summary: Option<String>,
    start: EventTime,
    end: EventTime,
}

#[derive(Deserialize, Debug)]
struct EventTime {
    #[serde(rename = "dateTime")]
    date_time: Option<String>,
}

#[derive(Debug)]
pub struct CalenderEventItem {
    pub summary: String,
    pub start: CalenderEventTime,
    pub end: CalenderEventTime,
}

#[derive(Debug)]
pub struct CalenderEventTime {
    pub date_time: DateTime<FixedOffset>,
}

pub async fn get_calenders(
    access_token: &str,
    config: &config::Config,
) -> Result<Vec<CalenderEventItem>> {
    let time_min = &config.start_date;
    let time_max = &config.end_date;
    let events_url = format!(
        "https://www.googleapis.com/calendar/v3/calendars/{}/events?singleEvents=true&timeMin={}&timeMax={}",
        config.calendar_id,
        encode(time_min),
        encode(time_max)
    );
    let mut headers = HeaderMap::new();
    let val = HeaderValue::from_str(&format!("Bearer {}", access_token));
    headers.insert(AUTHORIZATION, val.unwrap());

    let events: Events = Client::new()
        .get(&events_url)
        .headers(headers)
        .send()
        .await?
        .json()
        .await?;

    let calendar_events = events.items.into_iter().filter_map(|event| {
        if let (Some(summary), Some(start), Some(end)) =
            (event.summary, event.start.date_time, event.end.date_time)
        {
            let start_time = DateTime::<FixedOffset>::from_str(&start).unwrap();
            let end_time = DateTime::<FixedOffset>::from_str(&end).unwrap();
            Some(CalenderEventItem {
                summary,
                start: CalenderEventTime {
                    date_time: start_time,
                },
                end: CalenderEventTime {
                    date_time: end_time,
                },
            })
        } else {
            None
        }
    });

    let filtered_events: Vec<CalenderEventItem> = calendar_events
        .into_iter()
        .filter(|event| {
            let weekday = event.start.date_time.weekday();
            config.day_of_weeks.contains(&(weekday as u8))
        })
        .filter(|event| {
            let event_start_time: NaiveTime = event.start.date_time.time();
            event_start_time >= config.start_time && event_start_time < config.end_time
        })
        .collect();

    Ok(filtered_events)
}
