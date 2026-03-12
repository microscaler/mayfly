//! Tests for signal module: shutdown channel and config watcher.

use daemon::signal::{shutdown_channel, start_watcher};
use std::time::Duration;

#[test]
fn shutdown_channel_returns_receiver() {
    let rx = shutdown_channel().unwrap();
    // Receiver is valid; no signal sent so try_recv returns Empty (or would block on recv)
    let res = rx.recv_timeout(Duration::from_millis(10));
    assert!(res.is_err());
}

#[test]
fn start_watcher_accepts_path() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "").unwrap();
    let watcher = start_watcher(&path).unwrap();
    drop(watcher);
}
