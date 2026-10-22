pub mod prelude;

mod any_connection;
mod error;
pub use error::DatabaseError;
mod price_change;
mod schema;
mod station;
mod station_price_change;

use crate::any_connection::AnyConnection;

use dotenvy::dotenv;
use std::env;

use diesel::prelude::*;

pub fn establish_connection_sqlite() -> Result<AnyConnection, DatabaseError> {
    use diesel::connection::SimpleConnection;

    dotenv().ok();

    let database_url = env::var("DATABASE_URL")?;
    let mut conn = SqliteConnection::establish(&database_url)?;
    conn.batch_execute(concat!(
        "PRAGMA journal_mode = WAL; ",
        "PRAGMA synchronous = NORMAL; ",
        "PRAGMA wal_autocheckpoint = 1000; ",
        "PRAGMA busy_timeout = 250; ",
        "PRAGMA foreign_keys = ON; ",
    ))?;

    Ok(AnyConnection::Sqlite(conn))
}
