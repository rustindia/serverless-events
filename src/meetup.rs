use anyhow::Result;
use scraper::{Html, Selector};
use chrono::{ TimeZone, Utc};
use crate::DataRustIndiaEvents;
use crate::{fetch_html, add_event_to_array_events};
const MEETUP_BASE_URL: &str = "https://www.meetup.com";

pub fn fetch_meetup_upcoming_event(location: &str) -> Result<String> {
    let url = format!("{}/{}/events/", MEETUP_BASE_URL, location);
    fetch_html(&url)
}

fn fetch_meetup_past_event(location: &str) -> Result<String> {
    let url = format!("{}/{}/", MEETUP_BASE_URL, location);
    fetch_html(&url)
}

fn get_utc_date(event_date: String) -> String {
    Utc.datetime_from_str(&event_date, "%a, %b %d, %Y, %I:%M %p %Z").unwrap().to_string()
}

pub fn parse_meetup_fragment(body: String, loc_url: &str, array_events: &mut Vec<DataRustIndiaEvents>) {
    let document = Html::parse_document(&body);
    let uc_selector = Selector::parse(".eventCard").unwrap();

    if document.select(&uc_selector).next() != None {
        let upc_event = document.select(&uc_selector).next().unwrap();
        let upc_fragment = &upc_event.inner_html();
        let fragment = Html::parse_fragment(&upc_fragment);
        let uce_link_selector = Selector::parse(".eventCard--link").unwrap();
        let uce_a_element = fragment.select(&uce_link_selector).next().unwrap();
        let urlitem = &uce_a_element.value().attr("href").unwrap();
        let titleitem = uce_a_element.text().collect::<Vec<_>>().concat();
        let timeselector = Selector::parse("time").unwrap();
        let timeitem_element = fragment.select(&timeselector).next().unwrap();
        let timeitem = timeitem_element.text().collect::<Vec<_>>().concat();

        array_events.push(add_event_to_array_events(
            loc_url,
            titleitem,
            timeitem.clone(),
            format!("{}{}", MEETUP_BASE_URL, urlitem.to_string()),
            get_utc_date(timeitem),
        ));
        
    } else {
        let body = fetch_meetup_past_event(loc_url);
        let document = Html::parse_document(&body.unwrap());
        let selector = Selector::parse(r#"a[data-event-label="past-event-card-1"]"#).unwrap();
        let pastevent = document.select(&selector).next().unwrap();
        let urlitem = pastevent.value().attr("href").unwrap();
        let pe_fragment = pastevent.inner_html();
        let fragment = Html::parse_fragment(&pe_fragment);
        let timeselector = Selector::parse("time").unwrap();
        let titleselector = Selector::parse("span").unwrap();
        let timeitem = fragment.select(&timeselector).next().unwrap();
        let titleitem = fragment.select(&titleselector).next().unwrap();

        array_events.push(add_event_to_array_events(
            loc_url,
            titleitem.inner_html(),
            timeitem.inner_html(),
            urlitem.to_string(),
            get_utc_date(timeitem.inner_html()),
        ))
    }
}