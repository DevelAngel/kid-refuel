use crate::any_connection::AnyConnection;
use crate::error::DatabaseError;

use chrono::prelude::*;
use diesel::prelude::*;

#[derive(Debug)]
pub struct StationPriceChange {
    pub station_id: i32,
    pub name: String,
    pub addr: String,
    pub updated: DateTime<Utc>,
    pub price: [u8; 3],
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

    pub fn load_since(
        cutoff: DateTime<Utc>,
        conn: &mut AnyConnection,
    ) -> Result<Vec<Self>, DatabaseError> {
        match conn {
            AnyConnection::Sqlite(conn) => Self::load_since_sqlite(cutoff, conn),
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

    fn load_all_sqlite(conn: &mut SqliteConnection) -> Result<Vec<Self>, DatabaseError> {
        use crate::schema::price_changes::dsl::*;
        use crate::schema::stations::dsl::*;
        let rows = stations
            .inner_join(price_changes)
            .select((station_id, name, addr, updated, price))
            .order_by(updated.desc())
            .then_order_by(price.asc())
            .then_order_by(name.asc())
            .then_order_by(addr.asc())
            .load::<(i32, String, String, NaiveDateTime, i32)>(conn)?;

        rows.into_iter()
            .map(|(station_id_value, station_name, station_addr, changed_at, raw_price)| {
                Ok(Self {
                    station_id: station_id_value,
                    name: station_name,
                    addr: station_addr,
                    updated: Utc.from_utc_datetime(&changed_at),
                    price: convert_price(raw_price)?,
                })
            })
            .collect()
    }

    fn load_current_sqlite(conn: &mut SqliteConnection) -> Result<Vec<Self>, DatabaseError> {
        use crate::schema::price_changes::dsl::*;
        use crate::schema::stations::dsl::*;
        let p2 = diesel::alias!(crate::schema::price_changes as p2);
        let rows = stations
            .inner_join(price_changes)
            .filter(
                updated.nullable().eq(p2
                    .filter(station_id.eq(p2.field(station_id)))
                    .select(diesel::dsl::max(p2.field(updated)))
                    .single_value()),
            )
            .select((station_id, name, addr, updated, price))
            .order_by(name.asc())
            .then_order_by(addr.asc())
            .load::<(i32, String, String, NaiveDateTime, i32)>(conn)?;

        rows.into_iter()
            .map(|(station_id_value, station_name, station_addr, changed_at, raw_price)| {
                Ok(Self {
                    name: station_name,
                    station_id: station_id_value,
                    addr: station_addr,
                    updated: Utc.from_utc_datetime(&changed_at),
                    price: convert_price(raw_price)?,
                })
            })
            .collect()
    }

    fn load_since_sqlite(
        cutoff: DateTime<Utc>,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<Self>, DatabaseError> {
        use crate::schema::price_changes::dsl::*;
        use crate::schema::stations::dsl::*;

        let rows = stations
            .inner_join(price_changes)
            .filter(updated.ge(cutoff.naive_utc()))
            .select((station_id, name, addr, updated, price))
            .order_by(updated.asc())
            .then_order_by(name.asc())
            .then_order_by(addr.asc())
            .load::<(i32, String, String, NaiveDateTime, i32)>(conn)?;

        rows.into_iter()
            .map(|(station_id_value, station_name, station_addr, changed_at, raw_price)| {
                Ok(Self {
                    station_id: station_id_value,
                    name: station_name,
                    addr: station_addr,
                    updated: Utc.from_utc_datetime(&changed_at),
                    price: convert_price(raw_price)?,
                })
            })
            .collect()
    }



    fn load_station_sqlite(
        station_id_value: i32,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<Self>, DatabaseError> {
        use crate::schema::price_changes::dsl::*;
        use crate::schema::stations::dsl::{id, *};
        let rows = stations
            .inner_join(price_changes)
            .filter(id.eq(station_id_value))
            .select((station_id, name, addr, updated, price))
            .order_by(updated.desc())
            .load::<(i32, String, String, NaiveDateTime, i32)>(conn)?;

        rows.into_iter()
            .map(|(station_id_value, station_name, station_addr, changed_at, raw_price)| {
                Ok(Self {
                    station_id: station_id_value,
                    name: station_name,
                    addr: station_addr,
                    updated: Utc.from_utc_datetime(&changed_at),
                    price: convert_price(raw_price)?,
                })
            })
            .collect()
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

    #[test]
    fn loads_only_price_changes_on_or_after_the_cutoff() {
        use crate::price_change::NewPriceChange;
        use crate::station::NewStation;
        use diesel::connection::SimpleConnection;

        let mut sqlite = SqliteConnection::establish(":memory:").unwrap();
        sqlite
            .batch_execute(
                "CREATE TABLE stations (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    addr TEXT NOT NULL
                );
                CREATE TABLE price_changes (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    station_id INTEGER NOT NULL,
                    updated TIMESTAMP NOT NULL,
                    price INTEGER NOT NULL
                );",
            )
            .unwrap();
        let mut conn = AnyConnection::Sqlite(sqlite);
        let station_id = NewStation {
            name: "Alpha",
            addr: "1 Main Street",
        }
        .save(&mut conn)
        .unwrap();

        for (day, price) in [(1, 1234), (2, 1250)] {
            let updated = NaiveDate::from_ymd_opt(2026, 10, day)
                .unwrap()
                .and_hms_opt(12, 0, 0)
                .unwrap();
            NewPriceChange {
                station_id,
                updated: &updated,
                price,
            }
            .save(&mut conn)
            .unwrap();
        }

        let cutoff = Utc.with_ymd_and_hms(2026, 10, 2, 12, 0, 0).single().unwrap();
        let recent = StationPriceChange::load_since(cutoff, &mut conn).unwrap();

        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].station_id, station_id);
        assert_eq!(recent[0].price, [1, 25, 0]);
    }

}
