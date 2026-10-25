use chrono::{DateTime, Duration, TimeZone, Utc};
use cucumber::{World, given, then, when};
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use refuel_db::prelude::{AnyConnection, NewPriceChange, NewStation, StationPriceChange};
use std::collections::HashMap;
use std::path::PathBuf;

const UNKNOWN_STATION_ID: i32 = 9999;

#[derive(Debug, Default, World)]
#[world(init = Self::new)]
struct ComparisonWorld {
    _directory: Option<assert_fs::TempDir>,
    database_path: PathBuf,
    station_ids: HashMap<String, i32>,
    comparison: Vec<StationPriceChange>,
}

impl ComparisonWorld {
    fn new() -> Self {
        let directory = assert_fs::TempDir::new().expect("temporary directory should be created");
        let database_path = directory.path().join("refuel.sqlite");
        Self {
            _directory: Some(directory),
            database_path,
            ..Self::default()
        }
    }

    fn connection(&self) -> AnyConnection {
        let connection = SqliteConnection::establish(&self.database_path.to_string_lossy())
            .expect("test database should be reachable");
        AnyConnection::Sqlite(connection)
    }

    fn station_id(&mut self, name: &str) -> i32 {
        if let Some(id) = self.station_ids.get(name) {
            return *id;
        }
        let id = NewStation {
            name,
            addr: "1 Test Road",
        }
        .save(&mut self.connection())
        .expect("station should be stored");
        self.station_ids.insert(name.to_owned(), id);
        id
    }
}

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 25, 12, 0, 0)
        .single()
        .expect("fixed date should be valid")
}

fn format_price(price: [u8; 3]) -> String {
    format!("{}.{:02}{}", price[0], price[1], price[2])
}

fn split_names(names: &str) -> Vec<&str> {
    names.split(',').map(str::trim).collect()
}

#[given("the database is initialized with the application schema")]
async fn database_is_initialized(world: &mut ComparisonWorld) {
    let mut connection = SqliteConnection::establish(&world.database_path.to_string_lossy())
        .expect("test database should be created");
    for migration in [
        include_str!("../../db/migrations/2023-06-14-215000_create_stations/up.sql"),
        include_str!("../../db/migrations/2023-06-17-192209_create_price_changes/up.sql"),
        include_str!("../../db/migrations/2026-10-16-000000_harden_persistence/up.sql"),
        include_str!("../../db/migrations/2026-10-23-000000_add_station_coordinates/up.sql"),
    ] {
        connection
            .batch_execute(migration)
            .expect("migration should be applied");
    }
}

#[given(expr = "station {string} changed its price to {float} {int} days ago")]
async fn station_changed_its_price(
    world: &mut ComparisonWorld,
    name: String,
    price: f64,
    days_ago: i64,
) {
    let station_id = world.station_id(&name);
    let updated = (now() - Duration::days(days_ago)).naive_utc();
    NewPriceChange {
        station_id,
        updated: &updated,
        price: (price * 1000.0).round() as i32,
    }
    .save(&mut world.connection())
    .expect("price change should be stored");
}

#[when(expr = "the comparison is loaded for {string}")]
async fn comparison_is_loaded(world: &mut ComparisonWorld, names: String) {
    let ids = split_names(&names)
        .into_iter()
        .map(|name| {
            world
                .station_ids
                .get(name)
                .copied()
                .unwrap_or(UNKNOWN_STATION_ID)
        })
        .collect::<Vec<_>>();
    world.comparison = StationPriceChange::load_window(&ids, now(), &mut world.connection())
        .expect("comparison should be loadable");
}

#[then(expr = "the comparison contains only {string}")]
async fn comparison_contains_only(world: &mut ComparisonWorld, names: String) {
    let mut actual = world
        .comparison
        .iter()
        .map(|change| change.name.as_str())
        .collect::<Vec<_>>();
    actual.sort_unstable();
    actual.dedup();
    let mut expected = split_names(&names);
    expected.sort_unstable();

    assert_eq!(actual, expected);
}

#[then(expr = "the comparison for {string} has the prices {string}")]
async fn comparison_has_prices(world: &mut ComparisonWorld, name: String, expected: String) {
    let actual = world
        .comparison
        .iter()
        .filter(|change| change.name == name)
        .map(|change| format_price(change.price))
        .collect::<Vec<_>>()
        .join(", ");

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn gherkin_features() {
    ComparisonWorld::cucumber()
        .with_default_cli()
        .run_and_exit("tests/features/price_comparison.feature")
        .await;
}
