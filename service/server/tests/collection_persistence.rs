use cucumber::{given, then, when, World};
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use refuel_db::prelude::{AnyConnection, StationPriceChange};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Default, World)]
#[world(init = Self::new)]
struct CollectionWorld {
    _directory: Option<assert_fs::TempDir>,
    database_path: Option<PathBuf>,
    html_path: Option<PathBuf>,
}

impl CollectionWorld {
    fn new() -> Self {
        let directory = assert_fs::TempDir::new().expect("temporary directory should be created");
        let database_path = directory.path().join("refuel.sqlite");
        let html_path = directory.path().join("fixture.html");
        Self {
            _directory: Some(directory),
            database_path: Some(database_path),
            html_path: Some(html_path),
        }
    }

    fn database_path(&self) -> &PathBuf {
        self.database_path.as_ref().expect("database path should be initialized")
    }

    fn html_path(&self) -> &PathBuf {
        self.html_path.as_ref().expect("HTML path should be initialized")
    }
}

#[given("the simulator provides eight fuel stations")]
async fn simulator_provides_eight_stations(world: &mut CollectionWorld) {
    let fixture = refuel_sim::render_home().expect("simulator page should render");
    std::fs::write(world.html_path(), fixture).expect("simulator HTML should be written");
}

#[given("the database is initialized with the application schema")]
async fn database_is_initialized(world: &mut CollectionWorld) {
    let mut connection = SqliteConnection::establish(&world.database_path().to_string_lossy())
        .expect("test database should be created");
    connection
        .batch_execute(include_str!("../../db/migrations/2023-06-14-215000_create_stations/up.sql"))
        .expect("station schema should be created");
    connection
        .batch_execute(include_str!("../../db/migrations/2023-06-17-192209_create_price_changes/up.sql"))
        .expect("price schema should be created");
    connection
        .batch_execute(include_str!("../../db/migrations/2026-10-16-000000_harden_persistence/up.sql"))
        .expect("persistence constraints should be applied");
}

#[when("the real collector processes the simulator HTML")]
async fn collector_processes_simulator_html(world: &mut CollectionWorld) {
    let output = Command::new(env!("CARGO_BIN_EXE_refuel-server"))
        .args([
            "run-single",
            "--downloaded",
            world.html_path().to_str().expect("fixture path should be valid UTF-8"),
        ])
        .env("DATABASE_URL", world.database_path())
        .env("RUST_LOG", "refuel_server=debug,info")
        .output()
        .expect("collector process should start");
    assert!(
        output.status.success(),
        "collector failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[then("eight current station prices are queryable")]
async fn eight_current_station_prices_are_queryable(world: &mut CollectionWorld) {
    let connection = SqliteConnection::establish(&world.database_path().to_string_lossy())
        .expect("persisted database should be queryable");
    let mut connection = AnyConnection::Sqlite(connection);
    let current = StationPriceChange::load_current(&mut connection)
        .expect("persisted current prices should be queryable");
    assert_eq!(current.len(), 8);
}

#[then("MyESSO has its expected address and price")]
async fn my_esso_has_expected_address_and_price(world: &mut CollectionWorld) {
    let connection = SqliteConnection::establish(&world.database_path().to_string_lossy())
        .expect("persisted database should be queryable");
    let mut connection = AnyConnection::Sqlite(connection);
    let current = StationPriceChange::load_current(&mut connection)
        .expect("persisted current prices should be queryable");
    assert!(current.iter().any(|station| {
        station.name == "MyESSO"
            && station.addr == "Marienfelder Chaussee 171, 12349 Berlin"
            && station.price == [1, 78, 9]
    }));
}

#[tokio::test]
async fn gherkin_features() {
    CollectionWorld::cucumber()
        .with_default_cli()
        .run_and_exit("tests/features/collection_persistence.feature")
        .await;
}
