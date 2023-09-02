use anyhow::Result;
use bytes::Bytes;
use chrono::{Duration, TimeZone, Utc};
use http::StatusCode;
use serde::Serialize;
use spin_sdk::{
    http::{Request, Response},
    http_component,
    key_value::Store,
};
mod global;
mod hasgeek;
mod luma;
mod meetup;

use global::{fetch_html, add_event_to_array_events};

#[derive(Debug, Serialize)]
pub struct DataRustIndiaEvents {
    name: String,
    community: String,
    date: String,
    url: String,
    sort_date: String,
}

#[http_component]
fn handle_rust_india_events(_req: Request) -> Result<Response> {
    let store: Store = Store::open_default()?;

    let mut array_events: Vec<DataRustIndiaEvents> = vec![];
    let meetup_loc = ["rust-pune", "rust-hyderabad", "rustdelhi"];
    let hasgeek_loc = ["rustbangalore", "keralars", "rustchandigarh"];

    let last_fetch_at = match store.get("last_fetch_at") {
        Ok(value) => String::from_utf8_lossy(value.as_ref()).to_string(),
        Err(err) => {
            eprintln!("Error while fetching last_fetch_at: {}", err);
            let now_date = Utc::now().to_string();
            store.set("last_fetch_at", &now_date)?;
            now_date
        }
    };

    let date_now = Utc::now();
    let diff: Duration = date_now
        .signed_duration_since(Utc.datetime_from_str(&last_fetch_at, "%Y-%m-%d %H:%M:%S%.f UTC")?);
    let one_hour = Duration::minutes(1);

    if diff > one_hour {
        let now_date = Utc::now().to_string();
        store.set("last_fetch_at", &now_date)?;
        println!("set_last_fetch_at: {}", now_date);

        for loc_url in &meetup_loc {
            let body = meetup::fetch_meetup_upcoming_event(loc_url)?;
            meetup::parse_meetup_fragment(body, loc_url, &mut array_events)
        }

        for loc_url in &hasgeek_loc {
            let body = hasgeek::fetch_hasgeek_upcoming_data(loc_url)?;
            hasgeek::parse_hasgeek_fragment(body, loc_url, &mut array_events);
        }

        let body = luma::fetch_luma_data()?;
        let rust_mumbai_data = luma::parse_luma_fragment(body)?;
        array_events.push(rust_mumbai_data);

        // Sort     the objects based on the "date" field in descending order
        array_events.sort_by(|a, b| {
            let date_a = &a.sort_date;
            let date_b = &b.sort_date;
            date_b.cmp(&date_a)
        });

        // Serialize the sorted Vec<Value> back to a pretty printed JSON string
        let sorted_json_str = serde_json::to_string_pretty(&array_events).unwrap();

        store.set("events_data", &sorted_json_str)?;

        Ok(http::Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Some(Bytes::from(sorted_json_str)))?)
    } else {
        Ok(http::Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Some(Bytes::from(store.get("events_data")?)))?)
    }
}
