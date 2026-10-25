use chrono::NaiveDate;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use refuel_db::prelude::{AnyConnection, NewPriceChange, NewStation, Station, StationPriceChange};

fn legacy_connection() -> AnyConnection {
    let mut conn = SqliteConnection::establish(":memory:").unwrap();
    conn.batch_execute(include_str!(
        "../migrations/2023-06-14-215000_create_stations/up.sql"
    ))
    .unwrap();
    conn.batch_execute(include_str!(
        "../migrations/2023-06-17-192209_create_price_changes/up.sql"
    ))
    .unwrap();
    AnyConnection::Sqlite(conn)
}

fn connection() -> AnyConnection {
    let mut conn = legacy_connection();
    let AnyConnection::Sqlite(sqlite) = &mut conn;
    sqlite.batch_execute(include_str!("../migrations/2026-10-16-000000_harden_persistence/up.sql")).unwrap();
    sqlite.batch_execute(include_str!("../migrations/2026-10-23-000000_add_station_coordinates/up.sql")).unwrap();
    conn
}

fn save_observation(
    conn: &mut AnyConnection,
    station_id: i32,
    date: (i32, u32, u32),
    price: i32,
) -> i32 {
    let updated = NaiveDate::from_ymd_opt(date.0, date.1, date.2)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    NewPriceChange {
        station_id,
        updated: &updated,
        price,
    }
    .save(conn)
    .unwrap()
}

#[test]
fn repeated_station_and_observation_saves_are_idempotent() {
    let mut conn = connection();
    let station = NewStation {
        name: "Test Station",
        addr: "1 Test Road",
    }
    .save(&mut conn)
    .unwrap();
    let duplicate_station = NewStation {
        name: "Test Station",
        addr: "1 Test Road",
    }
    .save(&mut conn)
    .unwrap();

    assert_eq!(station, duplicate_station);

    let first = save_observation(&mut conn, station, (2026, 10, 1), 1234);
    let duplicate = save_observation(&mut conn, station, (2026, 10, 1), 9999);

    assert_eq!(first, duplicate);
    let history = StationPriceChange::load_station(station, &mut conn).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].price, [1, 23, 4]);
}

#[test]
fn price_queries_return_current_prices_and_station_history() {
    let mut conn = connection();
    let first_station = NewStation {
        name: "Alpha",
        addr: "1 Main Street",
    }
    .save(&mut conn)
    .unwrap();
    let second_station = NewStation {
        name: "Beta",
        addr: "2 Main Street",
    }
    .save(&mut conn)
    .unwrap();

    save_observation(&mut conn, first_station, (2026, 10, 1), 1234);
    save_observation(&mut conn, first_station, (2026, 10, 2), 1250);
    save_observation(&mut conn, second_station, (2026, 10, 1), 1400);

    let current = StationPriceChange::load_current(&mut conn).unwrap();
    assert_eq!(current.len(), 2);
    assert!(
        current
            .iter()
            .any(|item| item.name == "Alpha" && item.price == [1, 25, 0])
    );
    assert!(
        current
            .iter()
            .any(|item| item.name == "Beta" && item.price == [1, 40, 0])
    );

    let history = StationPriceChange::load_station(first_station, &mut conn).unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].price, [1, 25, 0]);
    assert_eq!(StationPriceChange::load_all(&mut conn).unwrap().len(), 3);
    assert_eq!(Station::load_all(&mut conn).unwrap().len(), 2);
}

#[test]
fn query_failures_are_returned_to_the_caller() {
    let mut conn = connection();
    let AnyConnection::Sqlite(sqlite) = &mut conn;
    sqlite.batch_execute("DROP TABLE stations").unwrap();

    assert!(Station::load_all(&mut conn).is_err());
}

#[test]
fn migration_deduplicates_existing_stations_and_observations() {
    let mut conn = legacy_connection();
    let AnyConnection::Sqlite(sqlite) = &mut conn;
    sqlite
        .batch_execute(
            "INSERT INTO stations (name, addr) VALUES
                ('Duplicate Station', '1 Main Street'),
                ('Duplicate Station', '1 Main Street');
             INSERT INTO price_changes (station_id, updated, price) VALUES
                (1, '2026-10-01 12:00:00', 1234),
                (2, '2026-10-01 12:00:00', 9999);",
        )
        .unwrap();
    sqlite
        .batch_execute(include_str!(
            "../migrations/2026-10-16-000000_harden_persistence/up.sql"
        ))
        .unwrap();
    sqlite.batch_execute(include_str!("../migrations/2026-10-23-000000_add_station_coordinates/up.sql")).unwrap();

    let stations = Station::load_all(&mut conn).unwrap();
    assert_eq!(stations.len(), 1);
    let station_id = stations[0].id.unwrap();
    let history = StationPriceChange::load_station(station_id, &mut conn).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].price, [1, 23, 4]);
}

#[test]
fn price_queries_return_complete_coordinates_and_fill_partial_pairs() {
    let mut conn = connection();
    let station_id = NewStation {
        name: "Alpha",
        addr: "1 Main Street",
    }
    .save(&mut conn)
    .unwrap();
    save_observation(&mut conn, station_id, (2026, 10, 1), 1234);

    {
        let AnyConnection::Sqlite(sqlite) = &mut conn;
        diesel::sql_query("UPDATE stations SET latitude = 51.0 WHERE id = ?")
            .bind::<diesel::sql_types::Integer, _>(station_id)
            .execute(sqlite)
            .unwrap();
    }

    Station::fill_missing_coordinates(station_id, 52.0, 13.7, &mut conn).unwrap();

    let current = StationPriceChange::load_current(&mut conn).unwrap();
    assert_eq!(current.len(), 1);
    assert_eq!(current[0].latitude, Some(52.0));
    assert_eq!(current[0].longitude, Some(13.7));

    let history = StationPriceChange::load_station(station_id, &mut conn).unwrap();
    assert_eq!(history[0].latitude, Some(52.0));
    assert_eq!(history[0].longitude, Some(13.7));
}


#[test]
fn coordinate_coverage_counts_only_stations_with_both_coordinates() {
    use refuel_db::prelude::CoordinateCoverage;

    let mut conn = connection();
    let first = NewStation { name: "Alpha", addr: "1 Main Street" }
        .save(&mut conn)
        .unwrap();
    NewStation { name: "Beta", addr: "2 Main Street" }
        .save(&mut conn)
        .unwrap();

    {
        let AnyConnection::Sqlite(sqlite) = &mut conn;
        diesel::sql_query("UPDATE stations SET latitude = 51.0, longitude = 13.7 WHERE id = ?")
            .bind::<diesel::sql_types::Integer, _>(first)
            .execute(sqlite)
            .unwrap();
        diesel::sql_query("UPDATE stations SET latitude = 51.1 WHERE name = 'Beta'")
            .execute(sqlite)
            .unwrap();
    }
    assert_eq!(
        CoordinateCoverage::load(&mut conn).unwrap(),
        CoordinateCoverage { total: 2, complete: 1 }
    );
    {
        let AnyConnection::Sqlite(sqlite) = &mut conn;
        diesel::sql_query("UPDATE stations SET longitude = 13.8 WHERE name = 'Beta'")
            .execute(sqlite)
            .unwrap();
    }

    assert_eq!(
        CoordinateCoverage::load(&mut conn).unwrap(),
        CoordinateCoverage { total: 2, complete: 2 }
    );
}

#[test]
fn coordinate_coverage_handles_an_empty_database() {
    use refuel_db::prelude::CoordinateCoverage;

    let mut conn = connection();
    assert_eq!(
        CoordinateCoverage::load(&mut conn).unwrap(),
        CoordinateCoverage { total: 0, complete: 0 }
    );
}
