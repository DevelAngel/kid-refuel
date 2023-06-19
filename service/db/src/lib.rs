pub mod prelude;

mod any_connection;
mod price_change;
mod schema;
mod station;

use crate::any_connection::AnyConnection;

use dotenvy::dotenv;
use std::env;

use diesel::prelude::*;

pub fn establish_connection_sqlite() -> AnyConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let conn = SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
    AnyConnection::Sqlite(conn)
}
