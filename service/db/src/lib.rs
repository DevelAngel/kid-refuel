pub mod prelude;

mod any_connection;
mod price_change;
mod schema;
mod station;
mod station_price_change;

use crate::any_connection::AnyConnection;

use dotenvy::dotenv;
use std::env;

use diesel::prelude::*;

pub fn establish_connection_sqlite() -> AnyConnection {
    use diesel::connection::SimpleConnection;

    dotenv().ok();

    let pragmas = concat!(
        "PRAGMA journal_mode = WAL; ",
        "PRAGMA synchronous = NORMAL; ",
        "PRAGMA wal_autocheckpoint = 1000; ",
        "PRAGMA wal_checkpoint(TRUNCATE); ",
        "PRAGMA busy_timeout = 250; ",
        "PRAGMA foreign_keys = ON; ",
    );

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut conn = SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
    conn.batch_execute(pragmas)
        .unwrap_or_else(|_| panic!("Error setting connection pragmas"));
    AnyConnection::Sqlite(conn)
}
