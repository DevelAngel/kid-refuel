use serde::{Deserialize, Serialize};

use chrono::prelude::*;

#[cfg(feature = "ssr")]
use refuel_db::prelude::RefuelStationPriceChange;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppRefuelStation {
    pub name: String,
    pub addr: String,
    pub price: [u8; 3],
    pub updated: DateTime<Utc>,
}

#[cfg(feature = "ssr")]
impl From<RefuelStationPriceChange> for AppRefuelStation {
    fn from(src: RefuelStationPriceChange) -> Self {
        Self {
            name: src.name,
            addr: src.addr,
            price: src.price,
            updated: src.updated,
        }
    }
}
