use crate::error::ParseError;
use refuel_db::prelude::*;

use chrono::{DateTime, Datelike, Local, TimeZone};
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use std::collections::VecDeque;
use tracing::debug;

type Result<T> = std::result::Result<T, ParseError>;

pub struct Station {
    pub name: String,
    pub addr: String,
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
    pub fn save(self, conn: &mut AnyConnection) -> std::result::Result<StationPriceChangeWithId, DatabaseError> {
        let station_id = NewStation {
            name: &self.station.name,
            addr: &self.station.addr,
        }
        .save(conn)?;

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
pub(crate) async fn parse(document: &Html) -> Result<VecDeque<StationPriceChange>> {
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
    let price_regex = Regex::new(
        r"(?P<eur>\d{1,2})[,.](?P<cent>\d{2})\s*(?P<subcent>\d)"
    )
    .map_err(|error| ParseError::InvalidPattern {
        pattern: "price",
        reason: error.to_string(),
    })?;

    let price_list = document
        .select(&price_list_selector)
        .next()
        .ok_or(ParseError::MissingPriceList)?;

    let mut stations = VecDeque::with_capacity(20);
    for element in price_list.select(&price_item_selector) {
        let parsed: Result<StationPriceChange> = (|| {
            let name = parse_text(&element, &name_selector, "station name")?;
            let addr = parse_text(&element, &address_selector, "station address")?;
            let updated = parse_updated(&element, &updated_selector, &updated_regex)?;
            let price = parse_price(&element, &price_selector, &price_regex)?;

            Ok(StationPriceChange {
                station: Station { name, addr },
                price_change: PriceChange { price, updated },
            })
        })();

        match parsed {
            Ok(station) => stations.push_back(station),
            Err(error) => debug!("skipping invalid price list entry: {error}"),
        }
    }

    Ok(stations)
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

fn parse_updated(element: &ElementRef<'_>, selector: &Selector, regex: &Regex) -> Result<DateTime<Local>> {

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
