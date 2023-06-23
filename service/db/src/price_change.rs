use crate::any_connection::AnyConnection;
use crate::schema::price_changes;
use crate::station::RefuelStation;

use chrono::prelude::*;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Identifiable, Associations, Debug)]
#[diesel(belongs_to(RefuelStation, foreign_key = station_id))]
#[diesel(table_name = price_changes)]
pub struct PriceChange {
    pub id: Option<i32>,
    pub station_id: i32,
    pub updated: NaiveDateTime,
    pub price: i32,
}

#[derive(Debug)]
pub struct RefuelStationPriceChange {
    pub name: String,
    pub addr: String,
    pub updated: DateTime<Utc>,
    pub price: [u8; 3],
}

impl RefuelStationPriceChange {
    pub fn load_all(conn: &mut AnyConnection) -> Vec<Self> {
        match conn {
            AnyConnection::Sqlite(conn) => Self::load_all_sqlite(conn),
        }
    }

    fn load_all_sqlite(conn: &mut SqliteConnection) -> Vec<Self> {
        use crate::schema::price_changes::dsl::*;
        use crate::schema::stations::dsl::*;
        let pc: Vec<_> = stations
            .inner_join(price_changes)
            .select((name, addr, updated, price))
            .get_results::<(String, String, NaiveDateTime, i32)>(conn)
            .unwrap();

        pc.into_iter()
            .map(|(curr_name, curr_addr, curr_updated, curr_price)| Self {
                name: curr_name,
                addr: curr_addr,
                updated: Utc.from_utc_datetime(&curr_updated),
                price: convert_price(curr_price),
            })
            .collect()
    }
}

#[derive(Insertable)]
#[diesel(table_name = price_changes)]
pub struct NewPriceChange<'a> {
    pub station_id: i32,
    pub updated: &'a NaiveDateTime,
    pub price: i32,
}

impl<'a> NewPriceChange<'a> {
    pub fn save(self, conn: &mut AnyConnection) -> Option<i32> {
        match conn {
            AnyConnection::Sqlite(conn) => self.save_sqlite(conn),
        }
    }

    fn save_sqlite(self, conn: &mut SqliteConnection) -> Option<i32> {
        use crate::schema::price_changes::dsl::*;
        let exists = price_changes
            .filter(station_id.eq(self.station_id))
            .filter(updated.eq(self.updated))
            .select(id)
            .first::<Option<i32>>(conn);
        match exists {
            Err(diesel::NotFound) => diesel::insert_into(price_changes)
                .values(self)
                .returning(id)
                .get_result(conn)
                .expect("Error saving new price change"),
            Ok(Some(other_id)) => Some(other_id),
            _ => None,
        }
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
