use serde::{Deserialize, Serialize};

use chrono::prelude::*;

#[cfg(feature = "ssr")]
use refuel_db::prelude::Station as DBStation;

#[cfg(feature = "ssr")]
use refuel_db::prelude::StationPriceChange as DBStationPriceChange;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Station {
    pub id: i32,
    pub name: String,
    pub addr: String,
}

#[cfg(feature = "ssr")]
impl From<DBStation> for Station {
    fn from(src: DBStation) -> Self {
        Self {
            id: src.id.expect("id set"),
            name: src.name,
            addr: src.addr,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StationPriceChange {
    pub name: String,
    pub addr: String,
    pub price: [u8; 3],
    pub updated: DateTime<Utc>,
}

#[cfg(feature = "ssr")]
impl From<DBStationPriceChange> for StationPriceChange {
    fn from(src: DBStationPriceChange) -> Self {
        Self {
            name: src.name,
            addr: src.addr,
            price: src.price,
            updated: src.updated,
        }
    }
}
