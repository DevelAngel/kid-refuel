use std::process::{Command, Output};

fn run_server(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_refuel-server"))
        .args(args)
        .output()
        .expect("server process should start")
}

#[test]
fn normal_mode_requires_a_fallback_position() {
    let output = run_server(&["--url", "https://example.com/prices"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains("missing fallback position"), "{stderr}");
}

#[test]
fn normal_mode_rejects_out_of_range_fallback_coordinates() {
    let output = run_server(&[
        "--url",
        "https://example.com/prices",
        "--fallback-position",
        "91,0",
    ]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains("invalid fallback position"), "{stderr}");
}
