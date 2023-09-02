use bytes::Bytes;
use anyhow::Result;
use crate::DataRustIndiaEvents;

pub fn fetch_html(url: &str) -> Result<String> {
    let res: http::Response<Option<Bytes>> = spin_sdk::outbound_http::send_request(
        http::Request::builder().method("GET").uri(url).body(None)?,
    )?;
    let body_bytes = res.body().clone().unwrap();
    Ok(String::from_utf8_lossy(&body_bytes).to_string())
}

pub fn add_event_to_array_events(
    loc_url: &str,
    name: String,
    date: String,
    url: String,
    sort_date: String,
) -> DataRustIndiaEvents {
    let event = match loc_url {
        "rust-pune" | "rust-hyderabad" | "rustdelhi" | "rustbangalore" | "kerala"
        | "rustchandigarh" => {
            let community = match loc_url {
                "rustbangalore" => "Rust Bangalore",
                "kerala" => "Rust Kerala",
                "rustchandigarh" => "Rust Chandigarh",
                "rust-pune" => "Rust Pune",
                "rust-hyderabad" => "Rust Hyderabad",
                "rustdelhi" => "Rust Delhi",
                _ => {
                    // Handle unrecognized loc_url here, e.g., set a default community.
                    // For example, you can use "Unknown Community".
                    "Unknown Community"
                }
            };

            DataRustIndiaEvents {
                name,
                community: community.to_string(),
                date,
                sort_date,
                url,
            }
        }
        _ => {
            // Handle unrecognized loc_url here, e.g., return a default event.
            // For example, you can return an event with default values.
            DataRustIndiaEvents {
                name: String::new(),
                community: String::new(),
                date: String::new(),
                sort_date: String::new(),
                url: String::new(),
            }
        }
    };
    event // Return the event
}