use cucumber::{given, then, when, World};
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use refuel_db::prelude::{AnyConnection, StationPriceChange};
use refuel_app::prelude::{CollectionDiagnostic, get_collection_diagnostics, record_collection_diagnostic};
use refuel_app::prelude::MAX_RESPONSE_EXCERPT_BYTES;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Default, World)]
#[world(init = Self::new)]
struct CollectionWorld {
    _directory: Option<assert_fs::TempDir>,
    database_path: Option<PathBuf>,
    html_path: Option<PathBuf>,
    response_excerpt: Option<String>,
}

impl CollectionWorld {
    fn new() -> Self {
        let directory = assert_fs::TempDir::new().expect("temporary directory should be created");
        let database_path = directory.path().join("refuel.sqlite");
        let html_path = directory.path().join("fixture.html");
        Self {
            _directory: Some(directory),
            database_path: Some(database_path),
            response_excerpt: None,
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

#[given("the Mehr-Tanken Dresden page is the input")]
async fn mehr_tanken_page_is_input(world: &mut CollectionWorld) {
    let fixture = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/https___www.mehr-tanken.de_regional-tanken_sachsen_dresden_01067_.html"
    );
    std::fs::copy(fixture, world.html_path())
        .expect("Mehr-Tanken HTML should be copied");
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

#[then("current Mehr-Tanken prices are queryable")]
async fn current_mehr_tanken_prices_are_queryable(world: &mut CollectionWorld) {
    let connection = SqliteConnection::establish(&world.database_path().to_string_lossy())
        .expect("persisted database should be queryable");
    let mut connection = AnyConnection::Sqlite(connection);
    let current = StationPriceChange::load_current(&mut connection)
        .expect("persisted current prices should be queryable");
    assert!(!current.is_empty());
}

#[then("JET has its expected address and price")]
async fn jet_has_expected_address_and_price(world: &mut CollectionWorld) {
    let connection = SqliteConnection::establish(&world.database_path().to_string_lossy())
        .expect("persisted database should be queryable");
    let mut connection = AnyConnection::Sqlite(connection);
    let current = StationPriceChange::load_current(&mut connection)
        .expect("persisted current prices should be queryable");
    assert!(current.iter().any(|station| {
        station.name == "JET"
            && station.addr == "Peschelstr. 36, 01139 Dresden"
            && station.price == [2, 12, 9]
    }));
}


#[when("twenty-one collection diagnostics are recorded")]
async fn twenty_one_diagnostics_are_recorded(_world: &mut CollectionWorld) {
    for index in 1..=21 {
        record_collection_diagnostic(CollectionDiagnostic {
            started_at: format!("2026-10-20T00:{index:02}:00+00:00"),
            source_url: url::Url::parse(&format!("https://example.test/{index}")).expect("valid test URL"),
            http_status: Some(http::StatusCode::OK),
            http_duration: Some(std::time::Duration::from_millis(5)),
            duration: std::time::Duration::from_millis(8),
            result: "success".to_owned(),
            prices_found: 1,
            rejected_entries: 0,
            error: None,
            response_excerpt: None,
        });
    }
}

#[then("the collection history contains only the latest twenty runs")]
async fn collection_history_contains_latest_twenty_runs(_world: &mut CollectionWorld) {
    let history = get_collection_diagnostics()
        .await
        .expect("collection history should be readable");

    assert_eq!(history.len(), 20);
    assert_eq!(history.first().map(|run| run.source_url.as_str()), Some("https://example.test/21"));
    assert_eq!(history.last().map(|run| run.source_url.as_str()), Some("https://example.test/2"));
}


#[when("a long response body is stored as an excerpt")]
async fn long_response_body_is_stored_as_excerpt(world: &mut CollectionWorld) {
    let body = format!("{}🧙", "a".repeat(MAX_RESPONSE_EXCERPT_BYTES - 2));
    world.response_excerpt = CollectionDiagnostic::response_excerpt(&body);
}

#[then("the response excerpt is bounded and valid UTF-8")]
async fn response_excerpt_is_bounded_and_valid_utf8(world: &mut CollectionWorld) {
    let excerpt = world.response_excerpt.as_ref()
        .expect("non-empty response should have an excerpt");

    assert!(excerpt.len() <= MAX_RESPONSE_EXCERPT_BYTES);
    assert!(excerpt.is_char_boundary(excerpt.len()));
}


#[tokio::test]
async fn gherkin_features() {
    CollectionWorld::cucumber()
        .with_default_cli()
        .run_and_exit("tests/features/collection_persistence.feature")
        .await;
}
