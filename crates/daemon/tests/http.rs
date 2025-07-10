use daemon::{self, config::Config};
use serial_test::serial;
use signal_hook::consts::SIGTERM;
use signal_hook::low_level::raise;
use std::time::Duration;

#[test]
#[serial]
fn health_endpoint_serves_ok() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "").unwrap();

    let cfg = Config::load(&path).unwrap();
    let daemon = daemon::init(cfg).unwrap();
    let handle = std::thread::spawn(move || daemon.run(false));
    std::thread::sleep(Duration::from_millis(100));

    let resp = reqwest::blocking::get("http://127.0.0.1:3000/__health").unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    assert_eq!(resp.text().unwrap(), "ok");

    raise(SIGTERM).unwrap();
    handle.join().unwrap().unwrap();
}

#[test]
#[serial]
fn metrics_endpoint_serves_text() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "").unwrap();

    let cfg = Config::load(&path).unwrap();
    let daemon = daemon::init(cfg).unwrap();
    let handle = std::thread::spawn(move || daemon.run(false));
    std::thread::sleep(Duration::from_millis(100));

    let resp = reqwest::blocking::get("http://127.0.0.1:3000/metrics").unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    assert!(resp.text().unwrap().contains("# no metrics"));

    raise(SIGTERM).unwrap();
    handle.join().unwrap().unwrap();
}
