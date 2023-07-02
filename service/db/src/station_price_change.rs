use crate::any_connection::AnyConnection;

use chrono::prelude::*;
use diesel::prelude::*;

#[derive(Debug)]
pub struct StationPriceChange {
    pub name: String,
    pub addr: String,
    pub updated: DateTime<Utc>,
    pub price: [u8; 3],
}

impl StationPriceChange {
    pub fn load_all(conn: &mut AnyConnection) -> Vec<Self> {
        match conn {
            AnyConnection::Sqlite(conn) => Self::load_all_sqlite(conn),
        }
    }

    pub fn load_current(conn: &mut AnyConnection) -> Vec<Self> {
        match conn {
            AnyConnection::Sqlite(conn) => Self::load_curr_sqlite(conn),
        }
    }

    pub fn load_station(id: i32, conn: &mut AnyConnection) -> Vec<Self> {
        match conn {
            AnyConnection::Sqlite(conn) => Self::load_station_sqlite(id, conn),
        }
    }

    fn load_all_sqlite(conn: &mut SqliteConnection) -> Vec<Self> {
        use crate::schema::price_changes::dsl::*;
        use crate::schema::stations::dsl::*;
        let pc: Vec<_> = stations
            .inner_join(price_changes)
            .select((name, addr, updated, price))
            .order_by(updated.desc())
            .then_order_by(price.asc())
            .then_order_by(name.asc())
            .then_order_by(addr.asc())
            .get_results::<(String, String, NaiveDateTime, i32)>(conn)
            .expect("sql failed");

        pc.into_iter()
            .map(|(cname, caddr, cupdated, cprice)| Self {
                name: cname,
                addr: caddr,
                updated: Utc.from_utc_datetime(&cupdated),
                price: convert_price(cprice),
            })
            .collect()
    }

    fn load_curr_sqlite(conn: &mut SqliteConnection) -> Vec<Self> {
        use crate::schema::price_changes::dsl::*;
        use crate::schema::stations::dsl::*;
        use diesel::dsl::max;
        let p2 = diesel::alias!(crate::schema::price_changes as p2);
        let pc: Vec<_> = stations
            .inner_join(price_changes)
            .filter(
                updated.nullable().eq(p2
                    .filter(station_id.eq(p2.field(station_id)))
                    .select(max(p2.field(updated)))
                    .single_value()),
            )
            .select((name, addr, updated, price))
            .order_by(name.asc())
            .then_order_by(addr.asc())
            .load::<(String, String, NaiveDateTime, i32)>(conn)
            .expect("sql failedd");

        pc.into_iter()
            .map(|(cname, caddr, cupdated, cprice)| Self {
                name: cname,
                addr: caddr,
                updated: Utc.from_utc_datetime(&cupdated),
                price: convert_price(cprice),
            })
            .collect()
    }

    fn load_station_sqlite(sid: i32, conn: &mut SqliteConnection) -> Vec<Self> {
        use crate::schema::price_changes::dsl::*;
        use crate::schema::stations::dsl::{id, *};
        let pc: Vec<_> = stations
            .inner_join(price_changes)
            .filter(id.eq(sid))
            .select((name, addr, updated, price))
            .order_by(updated.desc())
            .get_results::<(String, String, NaiveDateTime, i32)>(conn)
            .expect("sql failed");

        pc.into_iter()
            .map(|(cname, caddr, cupdated, cprice)| Self {
                name: cname,
                addr: caddr,
                updated: Utc.from_utc_datetime(&cupdated),
                price: convert_price(cprice),
            })
            .collect()
    }
}

fn convert_price(price: i32) -> [u8; 3] {
    //1234 -> [1, 23, 4]
    let eur = price / 1000; // 1234 / 1000 = 1
    let cent = (price / 10) % 100; // (1234 / 10) % 100 = 123 % 100 = 23
    let subcent = price % 10; // 1234 % 10 = 4

    let eur: u8 = eur.try_into().expect("conversion failed");
    let cent: u8 = cent.try_into().expect("conversion failed");
    let subcent: u8 = subcent.try_into().expect("conversion failed");

    [eur, cent, subcent]
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn price_test() {
        assert_eq!(convert_price(1234), [1, 23, 4]);
        assert_eq!(convert_price(4321), [4, 32, 1]);
        assert_eq!(convert_price(1879), [1, 87, 9]);
        assert_eq!(convert_price(1000), [1, 0, 0]);
        assert_eq!(convert_price(9999), [9, 99, 9]);
        assert_eq!(convert_price(0), [0, 0, 0]);
    }
}
