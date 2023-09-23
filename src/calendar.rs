use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client,
};
use serde::Deserialize;
use urlencoding::encode;

#[derive(Deserialize, Debug)]
struct Events {
    items: Vec<EventItem>,
}

#[derive(Deserialize, Debug)]
pub struct EventItem {
    pub summary: Option<String>,
    pub start: EventTime,
    pub end: EventTime,
}

#[derive(Deserialize, Debug)]
pub struct EventTime {
    #[serde(rename = "dateTime")]
    pub date_time: Option<String>,
}

pub async fn get_calenders(
    access_token: &str,
    calendar_id: &str,
    start: &str,
    end: &str,
) -> Result<Vec<EventItem>, reqwest::Error> {
    let time_min = start;
    let time_max = end;
    let events_url = format!(
        "https://www.googleapis.com/calendar/v3/calendars/{}/events?singleEvents=true&timeMin={}&timeMax={}",
        calendar_id,
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

    let filtered_events: Vec<EventItem> = events
        .items
        .into_iter()
        .filter(|event| {
            event.summary.is_some()
                && event.start.date_time.is_some()
                && event.end.date_time.is_some()
        })
        .collect();

    Ok(filtered_events)
}
