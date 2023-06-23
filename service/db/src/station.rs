use crate::any_connection::AnyConnection;
use crate::schema::stations;

use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = stations)]
pub struct RefuelStation {
    pub id: Option<i32>,
    pub name: String,
    pub addr: String,
}

#[derive(Insertable)]
#[diesel(table_name = stations)]
pub struct NewRefuelStation<'a> {
    pub name: &'a str,
    pub addr: &'a str,
}

impl<'a> NewRefuelStation<'a> {
    pub fn save(self, conn: &mut AnyConnection) -> Option<i32> {
        match conn {
            AnyConnection::Sqlite(conn) => self.save_sqlite(conn),
        }
    }

    fn save_sqlite(self, conn: &mut SqliteConnection) -> Option<i32> {
        use crate::schema::stations::dsl::*;
        let exists = stations
            .filter(name.eq(self.name))
            .filter(addr.eq(self.addr))
            .select(id)
            .first::<Option<i32>>(conn);
        match exists {
            Err(diesel::NotFound) => diesel::insert_into(stations)
                .values(self)
                .returning(id)
                .get_result(conn)
                .expect("Error saving new station"),
            Ok(Some(other_id)) => Some(other_id),
            _ => None,
        }
    }
}
