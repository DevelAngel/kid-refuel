use clap::Args;
use chrono_tz::Tz;
use refuel_app::prelude::FallbackPosition;
use url::Url;

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
}
