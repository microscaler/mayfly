use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use serial_test::serial;
use std::process::{Command, Stdio};
use std::time::Duration;

#[test]
#[serial]
fn debug_logs_emitted() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "").unwrap();

    let bin = assert_cmd::cargo::cargo_bin("tinkerbell");
    let child = Command::new(bin)
        .arg(&path)
        .env("RUST_LOG", "debug")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    std::thread::sleep(Duration::from_millis(200));
    let _ = kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM);
    let output = child.wait_with_output().unwrap();
    let logs = String::from_utf8_lossy(&output.stdout);
    assert!(logs.contains("DEBUG"), "logs missing debug level: {logs}");
}
