use crate::any_connection::AnyConnection;
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
    pub fn load_all(conn: &mut AnyConnection) -> Vec<Self> {
        match conn {
            AnyConnection::Sqlite(conn) => Self::load_all_sqlite(conn),
        }
    }

    fn load_all_sqlite(conn: &mut SqliteConnection) -> Vec<Self> {
        use crate::schema::stations::dsl::*;
        let s: Vec<_> = stations
            .select((id, name, addr))
            .order_by(name.asc())
            .then_order_by(addr.asc())
            .get_results::<(Option<i32>, String, String)>(conn)
            .expect("sql failed");

        s.into_iter()
            .map(|(cid, cname, caddr)| Self {
                id: cid,
                name: cname,
                addr: caddr,
            })
            .collect()
    }
}

#[derive(Insertable)]
#[diesel(table_name = stations)]
pub struct NewStation<'a> {
    pub name: &'a str,
    pub addr: &'a str,
}

impl<'a> NewStation<'a> {
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
