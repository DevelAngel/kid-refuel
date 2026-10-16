use std::process::{Command, Output};

fn run_collector(fixture: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_refuel-server"))
        .args([
            "run-single",
            "--downloaded",
            fixture,
            "--dry-run",
        ])
        .env("RUST_LOG", "refuel_server=debug,info")
        .output()
        .expect("collector process should start")
}
fn run_collector_content(html: &str) -> Output {
    use std::time::{SystemTime, UNIX_EPOCH};
 
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let path = std::env::temp_dir().join(format!("refuel-sim-{}-{nonce}.html", std::process::id()));
    std::fs::write(&path, html).expect("simulator HTML should be written");
    let output = run_collector(path.to_str().expect("temporary path should be valid UTF-8"));
    let _ = std::fs::remove_file(path);
    output
}
#[test]
fn parses_a_valid_price_observation() {
    let html = refuel_sim::render_home().expect("simulator page should render");
    let output = run_collector_content(&html);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "{stderr}");
    assert!(stderr.contains("prices fetched: 8"), "{stderr}");
    assert!(stderr.contains("MyESSO"), "{stderr}");
    assert!(stderr.contains("Marienfelder Chaussee 171"), "{stderr}");
}

#[test]
fn skips_malformed_entries_without_aborting_collection() {
    let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/malformed_prices.html");
    let output = run_collector(fixture);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "{stderr}");
    assert!(stderr.contains("prices fetched: 1"), "{stderr}");
    assert!(stderr.contains("Valid Station"), "{stderr}");
    assert!(!stderr.contains("Invalid Price Station"), "{stderr}");
    assert!(!stderr.contains("Invalid Timestamp Station"), "{stderr}");
}

#[test]
fn reports_a_missing_price_list_without_panicking() {
    let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/no_price_list.html");
    let output = run_collector(fixture);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(!stderr.contains("panicked at"), "{stderr}");
    assert!(stderr.contains("MissingPriceList"), "{stderr}");
}
