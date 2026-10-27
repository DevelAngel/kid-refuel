use clap::{Args, ValueEnum};
use chrono_tz::Tz;
use refuel_app::prelude::FallbackPosition;
use url::Url;

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum FuelType {
    E5,
    #[default]
    E10,
}

impl FuelType {
    pub fn label(self) -> &'static str {
        match self {
            Self::E5 => "E5",
            Self::E10 => "E10",
        }
    }
}


#[derive(Args)]
pub struct SourceConfig {
    #[arg(short, long, value_name = "URL", default_value = "http://localhost:8080")]
    /// URL of the price source
    pub url: Url,
    #[arg(long, value_name = "LATITUDE,LONGITUDE")]
    /// Fallback position used when the browser position is unavailable
    pub fallback_position: Option<FallbackPosition>,
    #[arg(long, value_name = "TIME_ZONE", default_value = "Europe/Berlin")]
    /// Time zone used by the price source
    pub source_time_zone: Tz,
    #[arg(long, value_enum, default_value_t = FuelType::E10)]
    /// Fuel type expected in the source price list
    pub fuel_type: FuelType,
}
