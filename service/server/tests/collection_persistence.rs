use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use refuel_db::prelude::{AnyConnection, StationPriceChange};
use std::process::Command;

use assert_fs::prelude::*;
use assert_fs::TempDir;

fn temporary_database(directory: &TempDir) -> AnyConnection {
    let path = directory.child("refuel.sqlite");
    let mut connection = SqliteConnection::establish(&path.path().to_string_lossy())
        .expect("test database should be created");
    connection
        .batch_execute(include_str!(
            "../../db/migrations/2023-06-14-215000_create_stations/up.sql"
        ))
        .expect("station schema should be created");
    connection
        .batch_execute(include_str!(
            "../../db/migrations/2023-06-17-192209_create_price_changes/up.sql"
        ))
        .expect("price schema should be created");
    connection
        .batch_execute(include_str!(
            "../../db/migrations/2026-10-16-000000_harden_persistence/up.sql"
        ))
        .expect("persistence constraints should be applied");

    AnyConnection::Sqlite(connection)
}

#[test]
fn collection_persists_simulated_prices_for_database_queries() {
    let directory = TempDir::new().expect("temporary directory should be created");
    let database_path = directory.child("refuel.sqlite");
    let mut connection = temporary_database(&directory);
    let fixture = refuel_sim::render_home().expect("simulator page should render");
    let html_path = directory.child("fixture.html");
    html_path
        .write_str(&fixture)
        .expect("simulator HTML should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_refuel-server"))
        .args([
            "run-single",
            "--downloaded",
            html_path
                .path()
                .to_str()
                .expect("fixture path should be valid UTF-8"),
        ])
        .env("DATABASE_URL", database_path.path())
        .env("RUST_LOG", "refuel_server=debug,info")
        .output()
        .expect("collector process should start");


    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");

    let current = StationPriceChange::load_current(&mut connection)
        .expect("persisted current prices should be queryable");
    assert_eq!(current.len(), 8);
    assert!(current.iter().any(|station| {
        station.name == "MyESSO"
            && station.addr == "Marienfelder Chaussee 171, 12349 Berlin"
            && station.price == [1, 78, 9]
    }));
    drop(connection);

}
