use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use refuel_db::prelude::{AnyConnection, StationPriceChange};
use std::process::Command;

fn temporary_database() -> (std::path::PathBuf, AnyConnection) {
    use std::time::{SystemTime, UNIX_EPOCH};

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let path = std::env::temp_dir().join(format!(
        "refuel-e2e-{}-{nonce}.sqlite",
        std::process::id()
    ));
    let mut connection = SqliteConnection::establish(&path.to_string_lossy())
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

    (path, AnyConnection::Sqlite(connection))
}

#[test]
fn collection_persists_simulated_prices_for_database_queries() {
    let (database_path, mut connection) = temporary_database();
    let fixture = refuel_sim::render_home().expect("simulator page should render");
    let nonce = database_path
        .file_stem()
        .and_then(|name| name.to_str())
        .expect("temporary database name should be valid UTF-8");
    let html_path = std::env::temp_dir().join(format!("{nonce}.html"));
    std::fs::write(&html_path, fixture).expect("simulator HTML should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_refuel-server"))
        .args([
            "run-single",
            "--downloaded",
            html_path.to_str().expect("fixture path should be valid UTF-8"),
        ])
        .env("DATABASE_URL", &database_path)
        .env("RUST_LOG", "refuel_server=debug,info")
        .output()
        .expect("collector process should start");

    let _ = std::fs::remove_file(html_path);

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

    let _ = std::fs::remove_file(&database_path);
    let _ = std::fs::remove_file(database_path.with_extension("sqlite-wal"));
    let _ = std::fs::remove_file(database_path.with_extension("sqlite-shm"));

}
