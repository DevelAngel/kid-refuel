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
