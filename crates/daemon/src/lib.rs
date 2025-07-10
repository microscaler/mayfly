//! Daemon runtime entry points.

pub mod config;
mod pal;
mod signal;

use crossbeam::channel::RecvTimeoutError;
use scheduler::Scheduler;
use std::time::Duration;
use tracing::instrument;

use config::Config;
use signal::shutdown_channel;

pub use pal::{DaemonEvent, emit as pal_emit, take_events};

/// Initialize the daemon and return a running instance.
#[instrument(skip(cfg))]
pub fn init(cfg: Config) -> anyhow::Result<Daemon> {
    tracing::info!("initializing daemon");
    Ok(Daemon { cfg })
}

/// Convenience wrapper to initialize and immediately run the daemon.
pub fn run(cfg: Config) -> anyhow::Result<()> {
    init(cfg)?.run()
}

/// Drive the scheduler until a shutdown signal is received.
///
/// This loops over [`Scheduler::run`] to process any queued tasks and waits for
/// a termination notification from [`signal::shutdown_channel`]. When the
/// scheduler becomes idle it blocks on the receiver for a short interval so
/// that the loop does not busy spin.
#[instrument(skip(sched))]
fn run_blocking(sched: &mut Scheduler) -> anyhow::Result<()> {
    let shutdown = shutdown_channel()?;
    loop {
        sched.run();
        if shutdown.try_recv().is_ok() {
            pal::emit(DaemonEvent::ShutdownBegin);
            break;
        }
        match shutdown.recv_timeout(Duration::from_millis(50)) {
            Ok(_) | Err(RecvTimeoutError::Disconnected) => {
                pal::emit(DaemonEvent::ShutdownBegin);
                break;
            }
            Err(RecvTimeoutError::Timeout) => {}
        }
    }
    Ok(())
}

/// Daemon state returned from [`init`].
pub struct Daemon {
    cfg: Config,
}

impl Daemon {
    /// Run the daemon until a termination signal is delivered.
    #[instrument(skip(self))]
    pub fn run(self) -> anyhow::Result<()> {
        tracing::info!("daemon running");
        // Watch for config changes (stub).
        let _watcher = signal::start_watcher(&self.cfg.config_path).ok();

        let mut sched = Scheduler::new();
        run_blocking(&mut sched)?;

        pal::emit(DaemonEvent::ShutdownComplete);
        tracing::info!("daemon shutdown complete");
        Ok(())
    }
}
