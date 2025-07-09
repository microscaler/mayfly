use daemon::{self, config::Config};
use serial_test::serial;
use signal_hook::consts::SIGTERM;
use signal_hook::low_level::raise;
use std::time::Duration;

#[test]
#[serial]
fn scheduler_exits_on_sigterm() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "").unwrap();

    let cfg = Config::load(&path).unwrap();
    let daemon = daemon::init(cfg).unwrap();
    let handle = std::thread::spawn(move || daemon.run());
    std::thread::sleep(Duration::from_millis(50));
    raise(SIGTERM).unwrap();
    handle.join().unwrap().unwrap();
}

#[test]
fn init_ok() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "").unwrap();
    let cfg = Config::load(&path).unwrap();
    daemon::init(cfg).unwrap();
}
