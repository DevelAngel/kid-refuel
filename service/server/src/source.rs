use crate::config::FuelType;
use crate::error::ParseError;
use crate::parse::{ParsedPrices, parse_mehr_tanken, parse_simulator};

use chrono_tz::Tz;
use scraper::Html;
use std::net::{Ipv4Addr, Ipv6Addr};
use url::{Host, Url};

#[derive(Clone, Copy, Debug)]
pub(crate) enum Source {
    Simulator,
    MehrTanken,
}

impl Source {
    pub(crate) fn from_url(url: &Url) -> Result<Self, ParseError> {
        match url.host() {
            Some(Host::Domain("localhost")) => Ok(Self::Simulator),
            Some(Host::Ipv4(ip)) if ip == Ipv4Addr::LOCALHOST => Ok(Self::Simulator),
            Some(Host::Ipv6(ip)) if ip == Ipv6Addr::LOCALHOST => Ok(Self::Simulator),
            Some(Host::Domain(domain))
                if domain == "mehr-tanken.de" || domain.ends_with(".mehr-tanken.de") =>
            {
                Ok(Self::MehrTanken)
            }
            _ => Err(ParseError::UnsupportedSource {
                url: url.to_string(),
            }),
        }
    }

    pub(crate) fn parse(
        self,
        document: &Html,
        source_time_zone: Tz,
        fuel_type: FuelType,
    ) -> Result<ParsedPrices, ParseError> {
        match self {
            Self::Simulator => parse_simulator(document, source_time_zone),
            Self::MehrTanken => parse_mehr_tanken(document, source_time_zone, fuel_type),
        }
    }
}
