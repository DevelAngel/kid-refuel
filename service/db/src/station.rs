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
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

impl Station {
    pub fn load_all(conn: &mut AnyConnection) -> Result<Vec<Self>, DatabaseError> {
        match conn {
            AnyConnection::Sqlite(conn) => {
                use crate::schema::stations::dsl::*;
                Ok(stations
                    .select((id, name, addr, latitude, longitude))
                    .order_by(name.asc())
                    .then_order_by(addr.asc())
                    .load::<Self>(conn)?)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoordinateCoverage {
    pub total: i64,
    pub complete: i64,
}

impl CoordinateCoverage {
    pub fn load(conn: &mut AnyConnection) -> Result<Self, DatabaseError> {
        match conn {
            AnyConnection::Sqlite(conn) => {
                use crate::schema::stations::dsl::*;

                let total = stations.count().get_result(conn)?;
                let complete = stations
                    .filter(latitude.is_not_null().and(longitude.is_not_null()))
                    .count()
                    .get_result(conn)?;

                Ok(Self { total, complete })
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
