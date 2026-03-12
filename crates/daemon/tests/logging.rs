use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use serial_test::serial;
use std::process::{Command, Stdio};
use std::time::Duration;

#[test]
#[serial]
fn daemon_accepts_rust_log_and_exits_on_sigterm() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "").unwrap();

    let bin = assert_cmd::cargo::cargo_bin("mayfly");
    let child = Command::new(bin)
        .arg(&path)
        .env("RUST_LOG", "debug")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    std::thread::sleep(Duration::from_millis(400));
    let _ = kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM);
    let output = child.wait_with_output().unwrap();
    // Daemon should exit when sent SIGTERM (success, 143, or killed by signal)
    let code = output.status.code();
    let ok = output.status.success()
        || code == Some(143)
        || code == Some(128 + 15)
        || (code.is_none() && !output.status.success());
    assert!(ok, "daemon should exit on SIGTERM: {:?}", output.status);
}
