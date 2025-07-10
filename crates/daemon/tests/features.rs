use serial_test::serial;
use std::process::Command;

fn build_with(features: &[&str]) {
    let mut cmd = Command::new("cargo");
    cmd.args(["build", "--package", "daemon"]);
    if !features.is_empty() {
        cmd.arg("--features");
        cmd.arg(features.join(","));
    }
    let status = cmd.status().expect("failed to run cargo");
    assert!(status.success());
}

#[test]
#[serial]
fn build_default() {
    build_with(&[]);
}

#[test]
#[serial]
fn build_ipc() {
    build_with(&["ipc"]);
}

#[test]
#[serial]
fn build_grpc() {
    build_with(&["grpc"]);
}

#[test]
#[serial]
fn build_a2a() {
    build_with(&["a2a"]);
}

#[test]
#[serial]
fn build_all() {
    build_with(&["ipc", "grpc", "a2a"]);
}
