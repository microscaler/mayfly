//! IPC interface for job submission over Unix sockets.
//!
//! This module is compiled only when the `ipc` feature is enabled. It provides a
//! simple stub service used in tests to ensure the optional interface builds
//! correctly.

use tracing::instrument;

/// Handle to the running IPC service.
#[derive(Debug)]
pub struct IpcService;

impl IpcService {
    /// Start the IPC service.
    #[instrument]
    pub fn start() -> Self {
        tracing::info!("ipc service started");
        Self
    }

    /// Shut down the IPC service.
    #[instrument]
    pub fn shutdown(self) {
        tracing::info!("ipc service stopped");
    }
}
