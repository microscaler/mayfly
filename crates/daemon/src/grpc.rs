//! gRPC interface for remote daemon control.
//!
//! This module is compiled only when the `grpc` feature is enabled. It exposes a
//! stub service that demonstrates how a real gRPC server could be wired into the
//! daemon.

use tracing::instrument;

/// Handle to the running gRPC service.
#[derive(Debug)]
pub struct GrpcService;

impl GrpcService {
    /// Start the gRPC service.
    #[instrument]
    pub fn start() -> Self {
        tracing::info!("grpc service started");
        Self
    }

    /// Shut down the gRPC service.
    #[instrument]
    pub fn shutdown(self) {
        tracing::info!("grpc service stopped");
    }
}
