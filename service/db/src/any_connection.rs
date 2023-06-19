use diesel::MultiConnection;

use diesel::prelude::*;

#[derive(MultiConnection)]
pub enum AnyConnection {
    Sqlite(SqliteConnection),
    //Postgresql(PgConnection),
}
