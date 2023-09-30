use std::collections::HashMap;

use chrono::Duration;

use crate::calendar::CalenderEventItem;

fn count_full_width_char(s: &str) -> usize {
    s.chars().filter(|c| !c.is_ascii()).count()
}

pub fn display(events: Vec<CalenderEventItem>) {
    println!("----------------------");
    let mut total_duration = Duration::zero();
    let mut duration_map: HashMap<String, Duration> = HashMap::new();

    let mut max_summary_len = 0;
    for event in &events {
        let duration = event.end.date_time - event.start.date_time;
        total_duration = total_duration + duration;
        let entry = duration_map
            .entry(event.summary.clone())
            .or_insert(Duration::seconds(0));
        *entry = *entry + duration;

        let adjusted_len = event.summary.len() + count_full_width_char(&event.summary);
        max_summary_len = std::cmp::max(max_summary_len, adjusted_len);
    }

    for (summary, duration) in &duration_map {
        let adjusted_width = max_summary_len - count_full_width_char(summary);
        println!(
            "{:<width$}: {:>3} min",
            summary,
            duration.num_minutes(),
            width = adjusted_width
        );
    }

    println!("======================");
    println!("合計時間: {:>3} min", total_duration.num_minutes());
}
