use anyhow::Result;
use bytes::Bytes;
use chrono::{DateTime, Duration, FixedOffset, TimeZone, Utc};
use http::StatusCode;
use scraper::{Html, Selector};
use serde::Serialize;
use serde_json::Value;
use spin_sdk::{
    http::{Request, Response},
    http_component,
    key_value::Store,
};

#[derive(Debug, Serialize)]
struct DataRustIndiaEvents {
    name: String,
    community: String,
    date: String,
    url: String,
    sort_date: String,
}

// Constants for the URLs
const MEETUP_BASE_URL: &str = "https://www.meetup.com";
const HASGEEK_BASE_URL: &str = "https://hasgeek.com";
const HASGEEK_PAST_URL: &str = "past.projects?page=1";
const LUMA_BASE_URL: &str = "https://lu.ma/rust-mumbai";

fn fetch_html(url: &str) -> Result<String> {
    let res: http::Response<Option<Bytes>> = spin_sdk::outbound_http::send_request(
        http::Request::builder().method("GET").uri(url).body(None)?,
    )?;

    let body_bytes = res.body().clone().unwrap();
    Ok(String::from_utf8_lossy(&body_bytes).to_string())
}

fn fetch_meetup_data(location: &str) -> Result<String> {
    let url = format!("{}/{}/", MEETUP_BASE_URL, location);
    fetch_html(&url)
}

fn fetch_hasgeek_upcoming_data(location: &str) -> Result<String> {
    let url = format!("{}/{}", HASGEEK_BASE_URL, location);
    fetch_html(&url)
}

fn fetch_hasgeek_past_data(location: &str) -> Result<String> {
    let url = format!("{}/{}/{}", HASGEEK_BASE_URL, location, HASGEEK_PAST_URL);
    fetch_html(&url)
}

fn fetch_luma_data() -> Result<String> {
    fetch_html(LUMA_BASE_URL)
}

#[http_component]
fn handle_serverless_events(_req: Request) -> Result<Response> {
    let store: Store = Store::open_default()?;

    let mut array_events: Vec<DataRustIndiaEvents> = vec![];
    let meetup_loc: [&str; 3] = ["rust-pune", "rust-hyderabad", "rustdelhi"];
    let hasgeek_loc: [&str; 3] = ["rustlangin", "keralars", "rustchandigarh"];

    let last_fetch_at = match store.get("last_fetch_at") {
        Ok(value) => String::from_utf8_lossy(value.as_ref()).to_string(),
        Err(err) => {
            eprintln!("Error while fetching last_fetch_at: {}", err);
            let now_date = Utc::now().to_string();
            store.set("last_fetch_at", &now_date)?;
            println!("set_last_fetch_at: {}", now_date);
            now_date
        }
    };

    let date_now = Utc::now();
    let diff: Duration = date_now.signed_duration_since(Utc.datetime_from_str(&last_fetch_at, "%Y-%m-%d %H:%M:%S%.f UTC")?);
    let one_hour = Duration::minutes(60);

    // println!("date_now: {}", date_now);
    // println!("last_fetch_at: {}", &last_fetch_at);
    
    if diff > one_hour {
        let now_date = Utc::now().to_string();
        store.set("last_fetch_at", &now_date)?;
        //println!("set_last_fetch_at: {}", now_date);

        for loc_url in &meetup_loc {
            //println!("{}", loc_url);
            let body = fetch_meetup_data(loc_url)?;

            let document = Html::parse_document(&body);
            let selector = Selector::parse(r#"a[data-event-label="past-event-card-1"]"#).unwrap();
            let pastevent = document.select(&selector).next().unwrap();
            let urlitem = pastevent.value().attr("href").unwrap();
            let pe_fragment = pastevent.inner_html();
            let fragment = Html::parse_fragment(&pe_fragment);
            let timeselector = Selector::parse("time").unwrap();
            let titleselector = Selector::parse("span").unwrap();
            let timeitem = fragment.select(&timeselector).next().unwrap();
            let titleitem = fragment.select(&titleselector).next().unwrap();

            match loc_url {
                &"rust-pune" | &"rust-hyderabad" | &"rustdelhi" => {
                    let community = match loc_url {
                        &"rust-pune" => "Rust Pune",
                        &"rust-hyderabad" => "Rust Hyderabad",
                        &"rustdelhi" => "Rust Delhi",
                        _ => panic!(),
                    };
                    let event = DataRustIndiaEvents {
                        name: titleitem.inner_html(),
                        community: community.to_string(),
                        date: timeitem.inner_html(),
                        sort_date: get_utc_date(timeitem.inner_html()),
                        url: urlitem.to_string(),
                    };
                    array_events.push(event);
                }
                _ => panic!(),
            }
        }

        for loc_url in &hasgeek_loc {
            //println!("hasgeek_loc, {}", loc_url);
            //hasgeek upcoming events

            let body = fetch_hasgeek_upcoming_data(loc_url)?;

            let document = Html::parse_document(&body);

            let selector = Selector::parse("a.card--upcoming").unwrap();

            if document.select(&selector).next() != None {

                let card_upcoming = document.select(&selector).next().unwrap();

                let cu_fragment = card_upcoming.inner_html();
                let fragment = Html::parse_fragment(&cu_fragment);

                let timeselector = Selector::parse("span.calendar__weekdays__dates__time").unwrap();

                if let Some(selected) = fragment.select(&timeselector).next() {
                    let timeitem = selected.inner_html();

                    let calselector =
                        Selector::parse("p.calendar__weekdays__dates__date--active").unwrap();
                    let cal_element = fragment.select(&calselector).next().unwrap();

                    let ec_html = cal_element.inner_html();
                    let ec_fragment = Html::parse_fragment(&ec_html);

                    let dn_selector = Selector::parse("span.calendar__weekdays__dates__date__name").unwrap();

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

                    let event_day = format!("{}, {} {}, {}, {}", dn_item, date_array[0], dd_item, date_array[1], timeitem);
                    let sort_date = format!("{}, {} {}, 12:00 AM", dn_item, dd_item, my_item);

                    match loc_url {
                        &"rustlangin" | &"kerala" | &"rustchandigarh" => {
                            let community = match loc_url {
                                &"rustlangin" => "Rust Bangalore",
                                &"kerala" => "Rust Kerala",
                                &"rustchandigarh" => "Rust Chandigarh",
                                _ => panic!(),
                            };
        
                            let event = DataRustIndiaEvents {
                                name: card_upcoming.value().attr("data-cy-title").unwrap().to_string(),
                                community: community.to_string(),
                                date: event_day,
                                url: format!("https://hasgeek.com{}",card_upcoming.value().attr("href").unwrap().to_string()),
                                sort_date: Utc.datetime_from_str(&sort_date, "%a, %d %b %Y, %I:%M %p").unwrap().to_string(),
                            };
                    
                            array_events.push(event);
                        }
                        _ => panic!(),
                    }
                }
            }

            //hasgeek past events
            let past_body = fetch_hasgeek_past_data(loc_url)?;

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

                match loc_url {
                    &"rustlangin" | &"kerala" | &"rustchandigarh" => {
                        let community = match loc_url {
                            &"rustlangin" => "Rust Bangalore",
                            &"kerala" => "Rust Kerala",
                            &"rustchandigarh" => "Rust Chandigarh",
                            _ => panic!(),
                        };
    
                        let event = DataRustIndiaEvents {
                            name: pe_title,
                            community: community.to_string(),
                            date: pe_date,
                            url: format!("https://hasgeek.com{}", urlitem),
                            sort_date: Utc
                                .datetime_from_str(&pe_sort_date, "%d %b %Y %H:%M:%S%.f UTC")
                                .unwrap()
                                .to_string(),
                        };
                
                        array_events.push(event);
                    }
                    _ => panic!(),
                }
            }
        }

        //println!("lu.ma_rust-mumbai");

        let body = fetch_luma_data()?;

        let document = Html::parse_document(&body);
        let title_selector = Selector::parse("h1.title").unwrap();
        let title_le = document.select(&title_selector).next().unwrap();
        let head = Selector::parse(r#"script[type="application/ld+json"]"#).unwrap();
        let head_le = document.select(&head).next().unwrap();

        let json_value: Value = serde_json::from_str(&head_le.inner_html())?;
        
        let me_formatted_date =if let Some(start_date) = json_value.get("startDate") {
            // Parse the input date string to DateTime<Utc>
            let parsed_datetime =DateTime::parse_from_rfc3339(&start_date.as_str().unwrap()).expect("Failed to parse the date string").with_timezone(&Utc);

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

        let rust_mumbai = DataRustIndiaEvents {
            name: title_le.inner_html(),
            community: "Rust Mumbai".to_string(),
            date: format!("{} {}", &me_formatted_date.clone().unwrap(), "IST"),
            url: me_url.unwrap(),
            sort_date: Utc.datetime_from_str(&me_formatted_date.unwrap(), "%a, %b %e, %Y, %I:%M %p").unwrap().to_string(),
        };

        array_events.push(rust_mumbai);

        // Sort     the objects based on the "date" field in descending order
        array_events.sort_by(|a, b| {
            let date_a = &a.sort_date;
            let date_b = &b.sort_date;
            date_b.cmp(&date_a)
        });

        // Serialize the sorted Vec<Value> back to a pretty printed JSON string
        let sorted_json_str = serde_json::to_string_pretty(&array_events).unwrap();
        
        store.set("events_data", &sorted_json_str)?;

        Ok(http::Response::builder().status(StatusCode::OK).body(Some(Bytes::from(sorted_json_str)))?)

    } else {
        Ok(http::Response::builder().status(StatusCode::OK).body(Some(Bytes::from(store.get("events_data")?)))?)
    }
}

fn get_utc_date(event_date: String) -> String {
    Utc.datetime_from_str(&event_date, "%a, %b %d, %Y, %I:%M %p %Z").unwrap().to_string()
}
