use crate::error::ParseError;
use refuel_db::prelude::*;

use chrono::{DateTime, Datelike, Local, TimeZone};
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use tracing::debug;

type Result<T> = std::result::Result<T, ParseError>;

pub(crate) struct ParsedPrices {
    pub stations: VecDeque<StationPriceChange>,
    pub rejected_entries: usize,
}

pub struct Station {
    pub name: String,
    pub addr: String,
    pub coordinates: Option<(f64, f64)>,
}

impl Station {
    fn valid_coordinates(latitude: f64, longitude: f64) -> Option<(f64, f64)> {
        ((-90.0..=90.0).contains(&latitude) && (-180.0..=180.0).contains(&longitude))
            .then_some((latitude, longitude))
    }
}

pub struct PriceChange {
    pub price: [u8; 3],
    pub updated: DateTime<Local>,
}

impl PriceChange {
    fn convert_to_subcent(&self) -> i32 {
        let price_eur = self.price[0] as i32;
        let price_cent = self.price[1] as i32;
        let price_subcent = self.price[2] as i32;
        price_eur * 1000 + price_cent * 10 + price_subcent
    }
}

pub struct StationPriceChange {
    pub station: Station,
    pub price_change: PriceChange,
}

pub struct StationPriceChangeWithId {
    pub id: i32,
    pub station: Station,
    pub price_change: PriceChange,
}

impl StationPriceChange {
    pub fn save(
        self,
        conn: &mut AnyConnection,
    ) -> std::result::Result<StationPriceChangeWithId, DatabaseError> {
        let station_id = NewStation {
            name: &self.station.name,
            addr: &self.station.addr,
        }
        .save(conn)?;
        if let Some((latitude, longitude)) = self.station.coordinates {
            refuel_db::prelude::Station::fill_missing_coordinates(
                station_id, latitude, longitude, conn,
            )?;
        }

        let updated = self.price_change.updated.naive_utc();
        NewPriceChange {
            station_id,
            updated: &updated,
            price: self.price_change.convert_to_subcent(),
        }
        .save(conn)?;
        Ok(StationPriceChangeWithId {
            id: station_id,
            station: self.station,
            price_change: self.price_change,
        })
    }
}
#[tracing::instrument(skip(document))]
pub(crate) async fn parse(document: &Html) -> Result<ParsedPrices> {
    let price_list_selector = selector(".PriceList")?;
    let price_item_selector = selector(".PriceList__item:not(.list-ad)")?;
    let name_selector = selector(".PriceList__itemTitle")?;
    let address_selector = selector(".PriceList__itemSubtitle")?;
    let updated_selector = selector(".PriceList__itemUpdated")?;
    let price_selector = selector(".PriceList__itemPrice")?;
    let updated_regex = Regex::new(
        r"(?P<day>\d{1,2})\.(?P<month>\d{1,2})\.(?:\s*(?P<year>\d{4}))?\s*(?P<hour>\d{1,2}):(?P<minute>\d{2})"
    )
    .map_err(|error| ParseError::InvalidPattern {
        pattern: "timestamp",
        reason: error.to_string(),
    })?;
    let price_regex = Regex::new(r"(?P<eur>\d{1,2})[,.](?P<cent>\d{2})\s*(?P<subcent>\d)")
        .map_err(|error| ParseError::InvalidPattern {
            pattern: "price",
            reason: error.to_string(),
        })?;

    if document.select(&price_list_selector).next().is_none() {
        return parse_mehr_tanken(document);
    }

    let price_list = document
        .select(&price_list_selector)
        .next()
        .ok_or(ParseError::MissingPriceList)?;

    let mut stations = VecDeque::with_capacity(20);
    let mut rejected_entries = 0;

    for element in price_list.select(&price_item_selector) {
        let parsed: Result<StationPriceChange> = (|| {
            let name = parse_text(&element, &name_selector, "station name")?;
            let addr = parse_text(&element, &address_selector, "station address")?;
            let updated = parse_updated(&element, &updated_selector, &updated_regex)?;
            let price = parse_price(&element, &price_selector, &price_regex)?;

            let attribute = |name: &str| {
                element
                    .value()
                    .attr(name)
                    .and_then(|value| value.trim().parse::<f64>().ok())
            };
            let coordinates = match (attribute("data-latitude"), attribute("data-longitude")) {
                (Some(latitude), Some(longitude)) => {
                    Station::valid_coordinates(latitude, longitude)
                }
                _ => None,
            };

            Ok(StationPriceChange {
                station: Station {
                    name,
                    addr,
                    coordinates,
                },
                price_change: PriceChange { price, updated },
            })
        })();

        match parsed {
            Ok(station) => stations.push_back(station),
            Err(error) => {
                rejected_entries += 1;
                debug!("skipping invalid price list entry: {error}");
            }
        }
    }

    Ok(ParsedPrices {
        stations,
        rejected_entries,
    })
}
fn parse_mehr_tanken(document: &Html) -> Result<ParsedPrices> {
    let station_selector = selector("a[href^=\"/tankstelle/\"]")?;
    let price_container_selector = selector("[class*=\"gasStationInfo_gas-price\"]")?;
    let info_container_selector = selector("[class*=\"gasStationInfo_gas-info\"]")?;
    let price_selector = selector("[class*=\"gasStationInfo_gas-price\"] > div:first-child")?;
    let updated_selector = selector("[class*=\"gasStationInfo_gas-price\"] > div:nth-child(2)")?;
    let name_selector = selector("[class*=\"gasStationInfo_gas-info\"] .va-headline")?;
    let address_selector = selector("[class*=\"gasStationInfo_gas-info\"] p.va-text")?;
    let updated_regex = Regex::new(
        r"(?P<day>\d{1,2})\.(?P<month>\d{1,2})\.(?:\s*(?P<year>\d{4}))?\s*(?P<hour>\d{1,2}):(?P<minute>\d{2})"
    )
    .map_err(|error| ParseError::InvalidPattern {
        pattern: "timestamp",
        reason: error.to_string(),
    })?;
    let price_regex = Regex::new(r"(?P<eur>\d{1,2})[,.](?P<cent>\d{2})").map_err(|error| {
        ParseError::InvalidPattern {
            pattern: "price",
            reason: error.to_string(),
        }
    })?;

    let coordinates_by_id = mehr_tanken_coordinates(document)?;

    let mut rejected_entries = 0;

    let mut stations = VecDeque::with_capacity(20);
    for element in document.select(&station_selector) {
        if element.select(&price_container_selector).next().is_none()
            || element.select(&info_container_selector).next().is_none()
        {
            continue;
        }

        let parsed: Result<StationPriceChange> = (|| {
            let name = parse_text(&element, &name_selector, "station name")?;
            let addr = parse_text(&element, &address_selector, "station address")?;
            let updated = parse_updated(&element, &updated_selector, &updated_regex)?;
            let value = parse_text(&element, &price_selector, "price")?;
            let captures =
                price_regex
                    .captures(&value)
                    .ok_or_else(|| ParseError::InvalidPrice {
                        value: value.clone(),
                    })?;
            let parse_part = |name: &str| {
                captures
                    .name(name)
                    .and_then(|value| value.as_str().parse::<u8>().ok())
            };
            let eur = parse_part("eur").ok_or_else(|| ParseError::InvalidPrice {
                value: value.clone(),
            })?;
            let cent = parse_part("cent").ok_or_else(|| ParseError::InvalidPrice {
                value: value.clone(),
            })?;

            let coordinates = element
                .value()
                .attr("href")
                .and_then(|href| href.trim_matches('/').rsplit('-').next())
                .and_then(|id| coordinates_by_id.get(id))
                .copied();

            Ok(StationPriceChange {
                station: Station {
                    name,
                    addr,
                    coordinates,
                },
                price_change: PriceChange {
                    price: [eur, cent, 9],
                    updated,
                },
            })
        })();

        match parsed {
            Ok(station) => stations.push_back(station),
            Err(error) => {
                rejected_entries += 1;
                debug!("skipping invalid Mehr-Tanken entry: {error}");
            }
        }
    }

    if stations.is_empty() {
        return Err(ParseError::MissingPriceList);
    }

    Ok(ParsedPrices {
        stations,
        rejected_entries,
    })
}

fn mehr_tanken_coordinates(document: &Html) -> Result<HashMap<String, (f64, f64)>> {
    let script_selector = selector("script#__NEXT_DATA__")?;
    let mut coordinates = HashMap::new();

    if let Some(script) = document.select(&script_selector).next() {
        let json = script.text().collect::<String>();
        match serde_json::from_str::<Value>(&json) {
            Ok(data) => collect_coordinates(&data, &mut coordinates),
            Err(error) => debug!("ignoring unreadable Mehr-Tanken station data: {error}"),
        }
    }

    Ok(coordinates)
}

fn collect_coordinates(value: &Value, coordinates: &mut HashMap<String, (f64, f64)>) {
    match value {
        Value::Object(object) => {
            if let (Some(id), Some(latitude), Some(longitude)) = (
                object.get("id").and_then(Value::as_str),
                object.get("lat").and_then(Value::as_f64),
                object.get("lon").and_then(Value::as_f64),
            ) && let Some(pair) = Station::valid_coordinates(latitude, longitude)
            {
                coordinates.insert(id.to_owned(), pair);
            }
            for value in object.values() {
                collect_coordinates(value, coordinates);
            }
        }
        Value::Array(items) => {
            for value in items {
                collect_coordinates(value, coordinates);
            }
        }
        _ => {}
    }
}

fn selector(value: &'static str) -> Result<Selector> {
    Selector::parse(value).map_err(|error| ParseError::InvalidSelector {
        selector: value,
        reason: error.to_string(),
    })
}

fn parse_text(
    element: &ElementRef<'_>,
    selector: &Selector,
    field: &'static str,
) -> Result<String> {
    let text = element
        .select(selector)
        .next()
        .map(|element| element.text().collect::<String>().trim().to_owned())
        .filter(|text| !text.is_empty())
        .ok_or(ParseError::MissingField { field })?;

    Ok(text)
}

fn parse_updated(
    element: &ElementRef<'_>,
    selector: &Selector,
    regex: &Regex,
) -> Result<DateTime<Local>> {
    let value = parse_text(element, selector, "timestamp")?;
    let captures = regex
        .captures(&value)
        .ok_or_else(|| ParseError::InvalidUpdated {
            value: value.clone(),
        })?;

    let parse_part = |name: &str| {
        captures
            .name(name)
            .and_then(|value| value.as_str().parse::<u32>().ok())
    };

    let day = parse_part("day").ok_or_else(|| ParseError::InvalidUpdated {
        value: value.clone(),
    })?;
    let month = parse_part("month").ok_or_else(|| ParseError::InvalidUpdated {
        value: value.clone(),
    })?;
    let hour = parse_part("hour").ok_or_else(|| ParseError::InvalidUpdated {
        value: value.clone(),
    })?;
    let minute = parse_part("minute").ok_or_else(|| ParseError::InvalidUpdated {
        value: value.clone(),
    })?;

    let now = Local::now();
    let year = match parse_part("year") {
        Some(year) => year as i32,
        None if month > now.month() && month - now.month() > 6 => now.year() - 1,
        None if now.month() > month && now.month() - month > 6 => now.year() + 1,
        None => now.year(),
    };

    Local
        .with_ymd_and_hms(year, month, day, hour, minute, 0)
        .single()
        .ok_or(ParseError::InvalidUpdated { value })
}

fn parse_price(element: &ElementRef<'_>, selector: &Selector, regex: &Regex) -> Result<[u8; 3]> {
    let value = parse_text(element, selector, "price")?;
    let captures = regex
        .captures(&value)
        .ok_or_else(|| ParseError::InvalidPrice {
            value: value.clone(),
        })?;

    let parse_part = |name: &str| {
        captures
            .name(name)
            .and_then(|value| value.as_str().parse::<u8>().ok())
    };

    let eur = parse_part("eur").ok_or_else(|| ParseError::InvalidPrice {
        value: value.clone(),
    })?;
    let cent = parse_part("cent").ok_or_else(|| ParseError::InvalidPrice {
        value: value.clone(),
    })?;
    let subcent = parse_part("subcent").ok_or_else(|| ParseError::InvalidPrice {
        value: value.clone(),
    })?;

    Ok([eur, cent, subcent])
}
