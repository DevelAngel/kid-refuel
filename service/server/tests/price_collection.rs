use cucumber::{World, given, then, when};
use std::path::PathBuf;
use std::process::{Command, Output};

#[derive(Debug, World)]
#[world(init = Self::new)]
struct CollectorWorld {
    _directory: assert_fs::TempDir,
    html_path: PathBuf,
    url: Option<String>,
    fuel_type: Option<String>,
    output: Option<Output>,
}

impl CollectorWorld {
    fn new() -> Self {
        let directory = assert_fs::TempDir::new().expect("temporary directory should be created");
        let html_path = directory.path().join("fixture.html");
        Self {
            _directory: directory,
            html_path,
            url: None,
            fuel_type: None,
            output: None,
        }
    }

    fn use_example(&self, relative_path: &str) {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path);
        std::fs::copy(source, &self.html_path).expect("fixture HTML should be copied");
    }

    fn output(&self) -> &Output {
        self.output.as_ref().expect("collector should have run")
    }

    fn log(&self) -> String {
        String::from_utf8_lossy(&self.output().stderr).into_owned()
    }
}

#[given("the Mehr-Tanken Dresden E5 page is the input")]
async fn mehr_tanken_e5_page_is_input(world: &mut CollectorWorld) {
    world.use_example("../../examples/mehr-tanken-dresden-E5.html");
}

#[given("its fuel type heading does not name a fuel type")]
async fn heading_names_no_fuel_type(world: &mut CollectorWorld) {
    let html = std::fs::read_to_string(&world.html_path).expect("page HTML should be readable");
    let html = html.replace(
        "Aktuelle Super (E5) Preisübersicht",
        "Aktuelle Kraftstoff Preisübersicht",
    );
    std::fs::write(&world.html_path, html).expect("page HTML should be written");
}

#[given("the simulator page is the input")]
async fn simulator_page_is_input(world: &mut CollectorWorld) {
    let html = refuel_sim::render_home().expect("simulator page should render");
    std::fs::write(&world.html_path, html).expect("simulator HTML should be written");
}

#[given("the page with malformed entries is the input")]
async fn page_with_malformed_entries_is_input(world: &mut CollectorWorld) {
    world.use_example("tests/fixtures/malformed_prices.html");
}

#[given("the page without a price list is the input")]
async fn page_without_price_list_is_input(world: &mut CollectorWorld) {
    world.use_example("tests/fixtures/no_price_list.html");
}

#[given(expr = "the source URL is {string}")]
async fn source_url_is(world: &mut CollectorWorld, url: String) {
    world.url = Some(url);
}

#[given(expr = "the configured fuel type is {string}")]
async fn configured_fuel_type_is(world: &mut CollectorWorld, fuel_type: String) {
    world.fuel_type = Some(fuel_type);
}

#[when("the collector dry-runs the page")]
async fn collector_dry_runs_the_page(world: &mut CollectorWorld) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_refuel-server"));
    command
        .args(["run-single", "--dry-run", "--downloaded"])
        .arg(&world.html_path)
        .env("RUST_LOG", "refuel_server=debug,info");
    if let Some(url) = &world.url {
        command.arg("--url").arg(url);
    }
    if let Some(fuel_type) = &world.fuel_type {
        command.arg("--fuel-type").arg(fuel_type);
    }
    world.output = Some(command.output().expect("collector process should start"));
}

#[then("the collector succeeds")]
async fn collector_succeeds(world: &mut CollectorWorld) {
    assert!(world.output().status.success(), "{}", world.log());
}

#[then("the collector fails")]
async fn collector_fails(world: &mut CollectorWorld) {
    assert!(!world.output().status.success(), "{}", world.log());
}

#[then(expr = "the log contains {string}")]
async fn log_contains(world: &mut CollectorWorld, text: String) {
    let log = world.log();
    assert!(log.contains(&text), "{log}");
}

#[then(expr = "the log does not contain {string}")]
async fn log_does_not_contain(world: &mut CollectorWorld, text: String) {
    let log = world.log();
    assert!(!log.contains(&text), "{log}");
}

#[tokio::test]
async fn gherkin_features() {
    CollectorWorld::cucumber()
        .with_default_cli()
        .run_and_exit("tests/features/price_collection.feature")
        .await;
}
