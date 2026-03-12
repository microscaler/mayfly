use crossbeam::channel::bounded;
use daemon::{self, DaemonEvent, config::Config, take_events};
use serial_test::serial;
use std::time::Duration;

fn run_daemon_until_shutdown(cfg: Config, shutdown_after_ms: u64) -> std::thread::JoinHandle<anyhow::Result<()>> {
    let (tx, rx) = bounded(1);
    let daemon = daemon::init(cfg).unwrap();
    let handle = std::thread::spawn(move || daemon.run_with_shutdown(rx, false));
    std::thread::sleep(Duration::from_millis(shutdown_after_ms));
    let _ = tx.send(());
    handle
}

#[test]
#[serial]
fn scheduler_exits_on_shutdown() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "").unwrap();

    let cfg = Config::load(&path).unwrap();
    let handle = run_daemon_until_shutdown(cfg, 100);
    handle.join().unwrap().unwrap();
    let _ = take_events();
}

#[test]
#[serial]
fn pal_events_logged_on_shutdown() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "").unwrap();

    let cfg = Config::load(&path).unwrap();
    let handle = run_daemon_until_shutdown(cfg, 100);
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
    let handle = run_daemon_until_shutdown(cfg, 100);
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
