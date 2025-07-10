use daemon::{self, DaemonEvent, config::Config, take_events};
use serial_test::serial;
use signal_hook::consts::{SIGINT, SIGTERM};
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
    let handle = std::thread::spawn(move || daemon.run(false));
    std::thread::sleep(Duration::from_millis(50));
    raise(SIGTERM).unwrap();
    handle.join().unwrap().unwrap();
    let _ = take_events();
}

#[test]
#[serial]
fn pal_events_logged_on_sigint() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "").unwrap();

    let cfg = Config::load(&path).unwrap();
    let daemon = daemon::init(cfg).unwrap();
    let handle = std::thread::spawn(move || daemon.run(false));
    std::thread::sleep(Duration::from_millis(50));
    raise(SIGINT).unwrap();
    handle.join().unwrap().unwrap();
    let events = take_events();
    assert!(events.contains(&DaemonEvent::WalFlushStart));
    assert!(events.contains(&DaemonEvent::MetricsStart));
    assert!(events.contains(&DaemonEvent::ShutdownBegin));
    assert!(events.contains(&DaemonEvent::ShutdownComplete));
}

#[test]
#[serial]
fn system_tasks_start() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "").unwrap();

    let cfg = Config::load(&path).unwrap();
    let daemon = daemon::init(cfg).unwrap();
    let handle = std::thread::spawn(move || daemon.run(false));
    std::thread::sleep(Duration::from_millis(50));
    raise(SIGTERM).unwrap();
    handle.join().unwrap().unwrap();
    let events = take_events();
    assert!(events.contains(&DaemonEvent::WalFlushStart));
    assert!(events.contains(&DaemonEvent::MetricsStart));
}

#[test]
fn init_ok() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "").unwrap();
    let cfg = Config::load(&path).unwrap();
    daemon::init(cfg).unwrap();
}
