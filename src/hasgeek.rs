use anyhow::Result;
use scraper::{Html, Selector};
use chrono::{TimeZone, Utc};
use crate::DataRustIndiaEvents;
use crate::{fetch_html, add_event_to_array_events};

const HASGEEK_BASE_URL: &str = "https://hasgeek.com";
const HASGEEK_PAST_URL: &str = "past.projects?page=1";

pub fn fetch_hasgeek_upcoming_data(location: &str) -> Result<String> {
    let url = format!("{}/{}", HASGEEK_BASE_URL, location);
    println!("{}",url);
    fetch_html(&url)
}

pub fn fetch_hasgeek_past_data(location: &str) -> Result<String> {
    let url = format!("{}/{}/{}", HASGEEK_BASE_URL, location, HASGEEK_PAST_URL);
    fetch_html(&url)
}

pub fn parse_hasgeek_fragment(
    body: String,
    loc_url: &str,
    array_events: &mut Vec<DataRustIndiaEvents>,
) {
    let document = Html::parse_document(&body);
    let upc_selector = Selector::parse(".upcoming").unwrap();

    if document.select(&upc_selector).next() != None {
        let selector = Selector::parse("a.card--upcoming").unwrap();
        let card_upcoming = document.select(&selector).next().unwrap();
        let cu_fragment = card_upcoming.inner_html();
        let fragment = Html::parse_fragment(&cu_fragment);
        let timeselector = Selector::parse("span.calendar__weekdays__dates__time").unwrap();

        if let Some(selected) = fragment.select(&timeselector).next() {
            let timeitem = selected.inner_html();
            let calselector = Selector::parse("p.calendar__weekdays__dates__date--active").unwrap();
            let cal_element = fragment.select(&calselector).next().unwrap();
            let ec_html = cal_element.inner_html();
            let ec_fragment = Html::parse_fragment(&ec_html);
            let dn_selector =
                Selector::parse("span.calendar__weekdays__dates__date__name").unwrap();

            let dn_item = if let Some(selected) = ec_fragment.select(&dn_selector).next() {
                selected.inner_html()
            } else {
                String::new()
            };

            let dd_selector = Selector::parse("span.calendar__weekdays__dates__date__day").unwrap();

            let dd_item = if let Some(selected) = ec_fragment.select(&dd_selector).next() {
                selected.inner_html()
            } else {
                String::new()
            };

            let my_selector = Selector::parse("span.calendar__month__name").unwrap();

            let my_item = if let Some(selected) = fragment.select(&my_selector).next() {
                selected.inner_html()
            } else {
                String::new()
            };

            let date_string = &my_item;
            let date_array: Vec<&str> = date_string.split_whitespace().collect();

            let event_day = format!(
                "{}, {} {}, {}, {}",
                dn_item, date_array[0], dd_item, date_array[1], timeitem
            );
            let sort_date = format!("{}, {} {}, 12:00 AM", dn_item, dd_item, my_item);

            array_events.push(add_event_to_array_events(
                loc_url,
                card_upcoming
                    .value()
                    .attr("data-cy-title")
                    .unwrap()
                    .to_string(),
                event_day,
                format!(
                    "{}{}",
                    HASGEEK_BASE_URL,
                    card_upcoming.value().attr("href").unwrap().to_string()
                ),
                Utc.datetime_from_str(&sort_date, "%a, %d %b %Y, %I:%M %p")
                    .unwrap()
                    .to_string(),
            ));
        }
    } else {
        //hasgeek past events
        println!("hasgeek_past events, {}", loc_url);
        let past_body = fetch_hasgeek_past_data(loc_url);

        parse_hasgeek_past_event_fragment(past_body.unwrap(), loc_url, array_events)
    }
}

pub fn parse_hasgeek_past_event_fragment(
    past_body: String,
    loc_url: &str,
    array_events: &mut Vec<DataRustIndiaEvents>,
) {
    let document = Html::parse_document(&past_body);

    let body_selector = Selector::parse("body").unwrap();
    let body_element = document.select(&body_selector).next().unwrap();

    let pe_fragment = body_element.inner_html();
    let fragment = Html::parse_fragment(&pe_fragment);
    let p_selector = Selector::parse("p").unwrap();

    let p_1 = fragment.select(&p_selector).nth(0).unwrap();

    if p_1.inner_html() != String::from("No past projects") {
        let p_2 = fragment.select(&p_selector).nth(1).unwrap();
        let p_2_fragment = Html::parse_fragment(&p_2.inner_html());
        let p_2_a_selector = Selector::parse("a").unwrap();
        let p_2_a_element = p_2_fragment.select(&p_2_a_selector).next().unwrap();

        let pe_date = p_1.inner_html();
        let pe_sort_date = format!("{} 00:00:0.000000000 UTC", pe_date);

        let pe_title = p_2_a_element.inner_html();
        let urlitem = p_2_a_element.value().attr("href").unwrap();

        array_events.push(add_event_to_array_events(
            loc_url,
            pe_title,
            pe_date,
            format!("{}{}", HASGEEK_BASE_URL, urlitem),
            Utc.datetime_from_str(&pe_sort_date, "%d %b %Y %H:%M:%S%.f UTC")
                .unwrap()
                .to_string(),
        ))
    }
}