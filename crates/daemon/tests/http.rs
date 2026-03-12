//! Tests for HTTP server: /__health, /metrics, and 404.

use daemon::http::HttpServer;
use std::net::TcpListener;

#[test]
fn http_health_returns_ok() {
    let addr = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap();
    let server = HttpServer::start(addr).unwrap();
    let url = format!("http://{addr}/__health");
    let res = reqwest::blocking::get(&url).unwrap();
    assert_eq!(res.status().as_u16(), 200);
    assert_eq!(res.text().unwrap(), "ok");
    server.shutdown();
}

#[test]
fn http_metrics_returns_placeholder() {
    let addr = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap();
    let server = HttpServer::start(addr).unwrap();
    let url = format!("http://{addr}/metrics");
    let res = reqwest::blocking::get(&url).unwrap();
    assert_eq!(res.status().as_u16(), 200);
    assert_eq!(res.text().unwrap(), "# no metrics");
    server.shutdown();
}

#[test]
fn http_unknown_path_returns_404() {
    let addr = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap();
    let server = HttpServer::start(addr).unwrap();
    let url = format!("http://{addr}/unknown");
    let res = reqwest::blocking::get(&url).unwrap();
    assert_eq!(res.status().as_u16(), 404);
    assert_eq!(res.text().unwrap(), "not found");
    server.shutdown();
}
