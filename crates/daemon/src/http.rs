//! Minimal HTTP server exposing daemon endpoints.
//!
//! This module hosts a tiny Hyper-based server used exclusively for
//! integration testing. It serves two endpoints:
//!
//! - `/metrics`  – placeholder Prometheus metrics output.
//! - `/__health` – liveness probe returning `200 OK` with body `ok`.
//!
//! The server runs on its own thread so that it doesn't block the
//! scheduler loop. It can be gracefully shut down using an
//! oneshot channel.

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use std::net::SocketAddr;
use std::thread::JoinHandle;
use tokio::sync::oneshot;
use tracing::instrument;

/// Default address the HTTP server listens on.
pub const DEFAULT_ADDR: &str = "127.0.0.1:3000";

/// Handle to the background HTTP server.
///
/// Dropping the handle does not stop the server; call \[`shutdown`\] to
/// terminate it and wait for the thread to exit.
pub struct HttpServer {
    shutdown: oneshot::Sender<()>,
    handle: JoinHandle<()>,
}

impl HttpServer {
    /// Spawn a new HTTP server bound to `addr`.
    ///
    /// The returned \[`HttpServer`\] can be used to signal shutdown once the
    /// daemon is terminating.
    #[instrument]
    pub fn start(addr: SocketAddr) -> anyhow::Result<Self> {
        let (tx, rx) = oneshot::channel();
        let handle = std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime");
            rt.block_on(async move {
                let make_svc = make_service_fn(|_| async {
                    Ok::<_, hyper::Error>(service_fn(handle_request))
                });
                let server = Server::bind(&addr).serve(make_svc);
                let server = server.with_graceful_shutdown(async {
                    let _ = rx.await;
                });
                if let Err(err) = server.await {
                    tracing::error!(%err, "http server error");
                }
            });
        });
        Ok(Self {
            shutdown: tx,
            handle,
        })
    }

    /// Gracefully stop the HTTP server, blocking until completion.
    #[instrument(skip(self))]
    pub fn shutdown(self) {
        let _ = self.shutdown.send(());
        let _ = self.handle.join();
    }
}

/// Dispatch incoming HTTP requests.
#[instrument]
async fn handle_request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match req.uri().path() {
        "/__health" => Ok(Response::new(Body::from("ok"))),
        "/metrics" => Ok(Response::new(Body::from("# no metrics"))),
        _ => {
            let mut res = Response::new(Body::from("not found"));
            *res.status_mut() = StatusCode::NOT_FOUND;
            Ok(res)
        }
    }
}
