//! A2A protocol support for message-based interaction.
//!
//! When the `a2a` feature is enabled this module provides a placeholder service
//! representing asynchronous agent-to-agent communication.

use tracing::instrument;

/// Handle to the running A2A service.
#[derive(Debug)]
pub struct A2aService;

impl A2aService {
    /// Start the A2A service.
    #[instrument]
    pub fn start() -> Self {
        tracing::info!("a2a service started");
        Self
    }

    /// Shut down the A2A service.
    #[instrument]
    pub fn shutdown(self) {
        tracing::info!("a2a service stopped");
    }
}
