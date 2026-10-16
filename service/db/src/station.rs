use crate::any_connection::AnyConnection;
use crate::error::DatabaseError;
use crate::schema::stations;

use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = stations)]
pub struct Station {
    pub id: Option<i32>,
    pub name: String,
    pub addr: String,
}

impl Station {
    pub fn load_all(conn: &mut AnyConnection) -> Result<Vec<Self>, DatabaseError> {
        match conn {
            AnyConnection::Sqlite(conn) => {
                use crate::schema::stations::dsl::*;
                Ok(stations
                    .select((id, name, addr))
                    .order_by(name.asc())
                    .then_order_by(addr.asc())
                    .load::<Self>(conn)?)
            }
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = stations)]
pub struct NewStation<'a> {
    pub name: &'a str,
    pub addr: &'a str,
}

impl<'a> NewStation<'a> {
    pub fn save(self, conn: &mut AnyConnection) -> Result<i32, DatabaseError> {
        match conn {
            AnyConnection::Sqlite(conn) => {
                use crate::schema::stations::dsl::*;
                diesel::insert_into(stations)
                    .values(&self)
                    .on_conflict_do_nothing()
                    .execute(conn)?;

                stations
                    .filter(name.eq(self.name))
                    .filter(addr.eq(self.addr))
                    .select(id)
                    .first::<Option<i32>>(conn)?
                    .ok_or(DatabaseError::MissingId("station"))
            }
        }
    }
}
