use crate::any_connection::AnyConnection;
use crate::error::DatabaseError;
use crate::schema::price_changes;
use crate::station::Station;

use chrono::prelude::*;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Identifiable, Associations, Debug)]
#[diesel(belongs_to(Station))]
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
    pub fn save(self, conn: &mut AnyConnection) -> Result<i32, DatabaseError> {
        match conn {
            AnyConnection::Sqlite(conn) => {
                use crate::schema::price_changes::dsl::*;
                diesel::insert_into(price_changes)
                    .values(&self)
                    .on_conflict_do_nothing()
                    .execute(conn)?;

                price_changes
                    .filter(station_id.eq(self.station_id))
                    .filter(updated.eq(self.updated))
                    .select(id)
                    .first::<Option<i32>>(conn)?
                    .ok_or(DatabaseError::MissingId("price change"))
            }
        }
    }
}
