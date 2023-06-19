use crate::error::ParseError;

use refuel_db::prelude::*;

use chrono::prelude::*;

use scraper::{Html, ElementRef, Selector};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::VecDeque;
use tokio::try_join;

use tracing::{debug, error};

type Result<T> = std::result::Result<T, ParseError>;

pub struct RefuelStation {
    pub name: String,
    pub addr: String,
}

pub struct PriceChange {
    pub price: [u8; 3],
    pub updated: DateTime<Utc>,
}

impl PriceChange {
    fn convert_to_subcent(&self) -> i32 {
        let price_eur = self.price[0] as i32;
        let price_cent = self.price[1] as i32;
        let price_subcent = self.price[2] as i32;
        price_eur * 1000 + price_cent * 10 + price_subcent
    }
}

pub struct RefuelStationPriceChange {
    pub station: RefuelStation,
    pub price_change: PriceChange,
}

pub struct RefuelStationPriceChangeWithId {
    pub id: i32,
    pub station: RefuelStation,
    pub price_change: PriceChange,
}

impl RefuelStationPriceChange {
    pub fn save(self, conn: &mut AnyConnection) -> RefuelStationPriceChangeWithId {
        let rs = NewRefuelStation {
            name: &self.station.name,
            addr: &self.station.addr,
        };
        let rs_id = rs.save(conn).unwrap();

        let pc = NewPriceChange {
            station_id: rs_id,
            updated: &self.price_change.updated.naive_utc(),
            price: self.price_change.convert_to_subcent(),
        };
        let _pc_id = pc.save(conn).unwrap();

        RefuelStationPriceChangeWithId {
            id: rs_id,
            station: self.station,
            price_change: self.price_change,
        }

    }
}

#[tracing::instrument(skip(document))]
pub(crate) async fn parse(document: &Html) -> Result<VecDeque<RefuelStationPriceChange>> {
    let selector_pricelist = Selector::parse(r#".PriceList"#).expect("invalid list selector");
    let selector_priceitem = Selector::parse(r#".PriceList__item:not(.list-ad)"#).expect("invalid list item selector");
    let selector_name = Selector::parse(r#".PriceList__itemTitle"#).expect("invalid name selector");
    let selector_addr = Selector::parse(r#".PriceList__itemSubtitle"#).expect("invalid addr selector");
    let selector_updated = Selector::parse(r#".PriceList__itemUpdated"#).expect("invalid updated selector");
    let selector_price = Selector::parse(r#".PriceList__itemPrice"#).expect("invalid price selector");

    let mut refuel_stations = VecDeque::with_capacity(20);
    let document = document.select(&selector_pricelist).next().expect("list not found");
    for elem in document.select(&selector_priceitem) {
        let name = parse_text(&elem, &selector_name);
        let addr = parse_text(&elem, &selector_addr);
        let updated = parse_updated(&elem, &selector_updated);
        let price = parse_price(&elem, &selector_price);

        match try_join!(name, addr, price, updated) {
            Ok((name, addr, price, updated)) => {
                let station = RefuelStationPriceChange {
                    station: RefuelStation { name, addr },
                    price_change: PriceChange { price, updated: updated.into() },
                };
                refuel_stations.push_back(station);
            }
            Err(err) => {
                match err {
                    ParseError::InvalidPriceError { html: _, regex: _ } |
                    ParseError::InvalidUpdatedError { html: _, regex: _ } => {
                        debug!("{err}");
                    }
                    _ => {
                        error!("{err}");
                    }
                }
            }
        }
    }

    Ok(refuel_stations)
}

#[tracing::instrument(skip(fragment))]
async fn parse_text<'a, 'b>(fragment: &ElementRef<'a>, selector: &'b Selector) -> Result<String> {
    lazy_static! {
        static ref REGEX: Regex = Regex::new(r"\b.+\b").expect("invalid text regex");
    }

    let text = fragment.select(selector).next().ok_or(ParseError::HtmlSelectError{
            html: fragment.inner_html(),
            selector: selector.clone(),
        })?;
    let text = text.inner_html();
    let text = REGEX.find_iter(&text).map(|mat| mat.as_str()).next().expect("text regex mismatch");
    Ok(text.to_owned())
}

#[tracing::instrument(skip(fragment))]
async fn parse_updated<'a, 'b>(fragment: &ElementRef<'a>, selector: &'b Selector) -> Result<DateTime<Local>> {
    lazy_static! {
        static ref REGEX: Regex = Regex::new(r#"(?P<d>\d{2})\.(?P<m>\d{2})\..(?P<h>\d{2}):(?P<min>\d{2})"#).expect("invalid updated regex");
        static ref REGEX_WS: Regex = Regex::new(r#"^\s*$"#).expect("invalid updated regex");
    }

    let updated = fragment.select(selector).next().ok_or(ParseError::HtmlSelectError{
            html: fragment.inner_html(),
            selector: selector.clone(),
        })?;
    let updated = updated.inner_html();

    if REGEX_WS.is_match(&updated) {
        return Err(ParseError::InvalidUpdatedError {
            html: updated,
            regex: REGEX_WS.clone(),
        });
    }

    let updated = REGEX.captures(&updated).ok_or(ParseError::RegexMismatchError{
            html: updated.to_owned(),
            regex: REGEX.clone(),
        })?;

    let year = if let Some(year) = updated.name("y") {
            year.as_str().parse()?
        } else {
            2023
        };
    let month = updated.name("m").expect("month missing in regex").as_str().parse().unwrap();
    let day = updated.name("d").expect("day missing in regex").as_str().parse().unwrap();
    let hour = updated.name("h").expect("day missing in regex").as_str().parse().unwrap();
    let min = updated.name("min").expect("day missing in regex").as_str().parse().unwrap();
    let sec = if let Some(sec) = updated.name("s") {
            sec.as_str().parse()?
        } else {
            0
        };

    // expect datetime shown in local time
    let datetime = Local.with_ymd_and_hms(year, month, day, hour, min, sec).unwrap();
    Ok(datetime)
}

#[tracing::instrument(skip(fragment))]
async fn parse_price<'a, 'b>(fragment: &ElementRef<'a>, selector: &'b Selector) -> Result<[u8; 3]> {
    lazy_static! {
        static ref REGEX_INVALID: Regex = Regex::new(r"[\-]\.[\-]{2}").expect("invalid invalid-price regex");
        static ref REGEX: Regex = Regex::new(r"\b(?P<eur>\d)\.(?P<cent>\d{2})\b(?s:.+)\b(?P<subcent>\d)\b").expect("invalid price regex");
    }

    let price = fragment.select(selector).next().ok_or(ParseError::HtmlSelectError{
            html: fragment.inner_html(),
            selector: selector.clone(),
        })?;

    let price = price.inner_html();

    if REGEX_INVALID.is_match(&price) {
        return Err(ParseError::InvalidPriceError { html: price, regex: REGEX_INVALID.clone() });
    }

    let price = REGEX.captures(&price).ok_or(ParseError::RegexMismatchError{
            html: price.to_owned(),
            regex: REGEX.clone(),
        })?;
    let eur: u8 = price.name("eur").expect("eur missing in regex").as_str().parse().expect("eur is no integer");
    let cent: u8 = price.name("cent").expect("cent missing in regex").as_str().parse().expect("cent is no integer");
    let subcent: u8 = price.name("subcent").expect("subcent missing in regex").as_str().parse().expect("subcent is no integer");
    assert!(subcent < 10);
    assert!(cent < 100);

    Ok([eur, cent, subcent])
}

