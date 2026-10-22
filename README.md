# kid-refuel

kid-refuel collects fuel prices, stores them in SQLite, and serves current prices
and station price history through a web application.

## Prerequisites

- The stable Rust toolchain. The repository's `rust-toolchain.toml` installs the
  `wasm32-unknown-unknown` target and Clippy component.
- SQLite command-line tools (`sqlite3`) for database inspection.
- Diesel CLI with SQLite support to apply database migrations:

  ```sh
  cargo install diesel_cli --no-default-features --features sqlite
  ```
- cargo-leptos to build the browser bundle and serve the web application:

  ```sh
  cargo install cargo-leptos --locked
  ```

## Build and test

Run commands from the workspace root:

```sh
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets
```

The workspace includes the server, frontend, database crate, and the simulator.
Server integration tests use the simulator as a library; they do not require
starting the simulator binary separately.

## Configure the database

The server reads the database location from `DATABASE_URL`. A `.env` file in
the workspace root is also loaded by the database crate. For local development,
create a `.env` file containing:

```dotenv
DATABASE_URL=refuel.db
```

Keep local database files and secrets out of version control.

Apply the database migrations from the workspace root:

```sh
diesel migration run
```

The migration directory is configured in `diesel.toml`. Run migrations before
starting the server against a new database. Back up an existing database before
applying schema changes.

## Run the server

For a local end-to-end run, start the bundled simulator in a separate terminal.
It listens on `http://localhost:8080` and provides sample fuel-price HTML:

```sh
cargo run -p refuel-sim
```

Then start the server in another terminal:

```sh
cargo leptos serve
```

Always build and start the web server through cargo-leptos (`just build`,
`just serve`, `just release`). Plain `cargo run` does not generate the browser
bundle and may reference differently named WASM files, so the browser requests
files that do not exist.

The server binds to the address configured by Leptos; verify the effective
address in the Leptos configuration before connecting. The collector uses the
URL configured by the server's `--url` option (default
`http://localhost:8080`) as its price source.

On startup, the server starts the web task and performs an initial collection.
If that collection fails, startup fails and the server exits. After a successful
startup, collection runs approximately once per hour, with a random jitter of
plus or minus ten minutes. A later collection failure is logged and the server
continues; the next attempt follows the normal schedule.

Press `Ctrl+C` to stop the server. It signals the collector to stop and waits
for the web server and collection task to shut down.

## Useful SQLite queries

Inspect tables and their schema:

```sh
sqlite3 refuel.db ".tables"
sqlite3 refuel.db ".schema"
```

Show all price observations with station details:

```sql
SELECT s.id, p.id, s.name, s.addr, p.price, p.updated
FROM stations AS s
JOIN price_changes AS p ON p.station_id = s.id;
```

Show all price observations for one station ID:

```sql
SELECT *
FROM price_changes
WHERE station_id = 1;
```

Show the latest observation for each station, including its name and address:

```sql
SELECT s.id, p.id, p.updated, p.price, s.name, s.addr
FROM stations AS s
JOIN price_changes AS p ON s.id = p.station_id
WHERE p.updated = (
    SELECT MAX(p2.updated)
    FROM price_changes AS p2
    WHERE p2.station_id = p.station_id
)
ORDER BY s.id;
```
