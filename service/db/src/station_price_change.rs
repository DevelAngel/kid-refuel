use crate::any_connection::AnyConnection;
use crate::error::DatabaseError;

use chrono::prelude::*;
use diesel::prelude::*;

const WINDOW_WEEKS: i64 = 5;

type Row = (
    i32,
    String,
    String,
    NaiveDateTime,
    i32,
    Option<f64>,
    Option<f64>,
);

#[derive(Debug)]
pub struct StationPriceChange {
    pub station_id: i32,
    pub name: String,
    pub addr: String,
    pub updated: DateTime<Utc>,
    pub price: [u8; 3],
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

impl StationPriceChange {
    pub fn load_all(conn: &mut AnyConnection) -> Result<Vec<Self>, DatabaseError> {
        match conn {
            AnyConnection::Sqlite(conn) => Self::load_all_sqlite(conn),
        }
    }

    pub fn load_current(conn: &mut AnyConnection) -> Result<Vec<Self>, DatabaseError> {
        match conn {
            AnyConnection::Sqlite(conn) => Self::load_current_sqlite(conn),
        }
    }

    pub fn load_station(
        station_id_value: i32,
        conn: &mut AnyConnection,
    ) -> Result<Vec<Self>, DatabaseError> {
        match conn {
            AnyConnection::Sqlite(conn) => Self::load_station_sqlite(station_id_value, conn),
        }
    }

    /// Loads the changes of the given stations within the last five weeks,
    /// each station starting with the change that was valid at the window start.
    pub fn load_window(
        station_ids: &[i32],
        now: DateTime<Utc>,
        conn: &mut AnyConnection,
    ) -> Result<Vec<Self>, DatabaseError> {
        match conn {
            AnyConnection::Sqlite(conn) => Self::load_window_sqlite(station_ids, now, conn),
        }
    }

    fn from_row(
        (station_id, name, addr, updated, price, latitude, longitude): Row,
    ) -> Result<Self, DatabaseError> {
        Ok(Self {
            station_id,
            name,
            addr,
            updated: Utc.from_utc_datetime(&updated),
            price: convert_price(price)?,
            latitude,
            longitude,
        })
    }

    fn load_all_sqlite(conn: &mut SqliteConnection) -> Result<Vec<Self>, DatabaseError> {
        use crate::schema::price_changes::dsl::*;
        use crate::schema::stations::dsl::*;

        stations
            .inner_join(price_changes)
            .select((station_id, name, addr, updated, price, latitude, longitude))
            .order_by(updated.desc())
            .then_order_by(price.asc())
            .then_order_by(name.asc())
            .then_order_by(addr.asc())
            .load::<Row>(conn)?
            .into_iter()
            .map(Self::from_row)
            .collect()
    }

    fn load_current_sqlite(conn: &mut SqliteConnection) -> Result<Vec<Self>, DatabaseError> {
        use crate::schema::price_changes::dsl::*;
        use crate::schema::stations::dsl::*;

        let p2 = diesel::alias!(crate::schema::price_changes as p2);
        stations
            .inner_join(price_changes)
            .filter(
                updated.nullable().eq(p2
                    .filter(station_id.eq(p2.field(station_id)))
                    .select(diesel::dsl::max(p2.field(updated)))
                    .single_value()),
            )
            .select((station_id, name, addr, updated, price, latitude, longitude))
            .order_by(name.asc())
            .then_order_by(addr.asc())
            .load::<Row>(conn)?
            .into_iter()
            .map(Self::from_row)
            .collect()
    }

    fn load_station_sqlite(
        station_id_value: i32,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<Self>, DatabaseError> {
        use crate::schema::price_changes::dsl::*;
        use crate::schema::stations::dsl::{id as station_pk, *};

        stations
            .inner_join(price_changes)
            .filter(station_pk.eq(station_id_value))
            .select((station_id, name, addr, updated, price, latitude, longitude))
            .order_by(updated.desc())
            .load::<Row>(conn)?
            .into_iter()
            .map(Self::from_row)
            .collect()
    }

    fn load_window_sqlite(
        station_ids: &[i32],
        now: DateTime<Utc>,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<Self>, DatabaseError> {
        use crate::schema::price_changes::dsl::*;
        use crate::schema::stations::dsl::*;

        let since = (now - chrono::Duration::weeks(WINDOW_WEEKS)).naive_utc();
        let mut rows = Vec::new();
        for &requested in station_ids {
            let at_start = stations
                .inner_join(price_changes)
                .filter(station_id.eq(requested))
                .filter(updated.lt(since))
                .select((station_id, name, addr, updated, price, latitude, longitude))
                .order_by(updated.desc())
                .first::<Row>(conn)
                .optional()?;
            let within = stations
                .inner_join(price_changes)
                .filter(station_id.eq(requested))
                .filter(updated.ge(since))
                .select((station_id, name, addr, updated, price, latitude, longitude))
                .order_by(updated.asc())
                .load::<Row>(conn)?;
            rows.extend(at_start);
            rows.extend(within);
        }

        rows.into_iter().map(Self::from_row).collect()
    }
}

fn convert_price(price: i32) -> Result<[u8; 3], DatabaseError> {
    if price < 0 {
        return Err(DatabaseError::InvalidPrice(price));
    }

    let eur = u8::try_from(price / 1000).map_err(|_| DatabaseError::InvalidPrice(price))?;
    let cent = ((price / 10) % 100) as u8;
    let subcent = (price % 10) as u8;

    Ok([eur, cent, subcent])
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn converts_price_to_euros_cents_and_subcent() {
        assert_eq!(convert_price(1234).unwrap(), [1, 23, 4]);
        assert_eq!(convert_price(4321).unwrap(), [4, 32, 1]);
        assert_eq!(convert_price(1879).unwrap(), [1, 87, 9]);
        assert_eq!(convert_price(1000).unwrap(), [1, 0, 0]);
        assert_eq!(convert_price(99999).unwrap(), [99, 99, 9]);
        assert_eq!(convert_price(0).unwrap(), [0, 0, 0]);
    }

    #[test]
    fn rejects_negative_prices() {
        assert!(matches!(
            convert_price(-1),
            Err(DatabaseError::InvalidPrice(-1))
        ));
    }
}
