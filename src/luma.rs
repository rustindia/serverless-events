use anyhow::Result;
use scraper::{Html, Selector};
use serde_json::Value;
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use crate::DataRustIndiaEvents;
use crate::fetch_html;
const LUMA_BASE_URL: &str = "https://lu.ma/rust-mumbai-2";

pub fn fetch_luma_data() -> Result<String> {
    fetch_html(LUMA_BASE_URL)
}

pub fn parse_luma_fragment(body: String) -> Result<DataRustIndiaEvents> {
    let document = Html::parse_document(&body);
    let title_selector = Selector::parse("h1.title").unwrap();
    let title_le = document.select(&title_selector).next().unwrap();
    let head = Selector::parse(r#"script[type="application/ld+json"]"#).unwrap();
    let head_le = document.select(&head).next().unwrap();

    let json_value: Value = serde_json::from_str(&head_le.inner_html())?;

    let me_formatted_date = if let Some(start_date) = json_value.get("startDate") {
        // Parse the input date string to DateTime<Utc>
        let parsed_datetime = DateTime::parse_from_rfc3339(&start_date.as_str().unwrap())
            .expect("Failed to parse the date string")
            .with_timezone(&Utc);

        // Define the Indian Standard Time (IST) offset from UTC (+5 hours and 30 minutes)
        let ist_offset = FixedOffset::east_opt(5 * 3600 + 30 * 60).expect("Invalid time offset");

        // Convert the DateTime<Utc> to IST DateTime<FixedOffset>
        let ist_datetime = parsed_datetime.with_timezone(&ist_offset);

        // Format the date in the desired format
        let formatted_date = ist_datetime.format("%a, %b %e, %Y, %I:%M %p");

        Some(formatted_date.to_string())
    } else {
        None
    };

    let me_url: Option<String> = if let Some(url) = json_value.get("@id") {
        Some(url.to_string())
    } else {
        None
    };

    Ok(DataRustIndiaEvents {
        name: title_le.inner_html(),
        community: "Rust Mumbai".to_string(),
        date: format!("{} {}", &me_formatted_date.clone().unwrap(), "IST"),
        url: me_url.unwrap(),
        sort_date: Utc
            .datetime_from_str(&me_formatted_date.unwrap(), "%a, %b %e, %Y, %I:%M %p")
            .unwrap()
            .to_string(),
    })
}