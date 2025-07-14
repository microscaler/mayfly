//! Daemon runtime entry points.

#[cfg(feature = "a2a")]
pub mod a2a;
pub mod config;
#[cfg(feature = "grpc")]
pub mod grpc;
mod http;
#[cfg(feature = "ipc")]
pub mod ipc;
mod pal;
mod signal;

use crossbeam::channel::RecvTimeoutError;
use scheduler::{Scheduler, SystemCall, TaskContext};
use std::net::SocketAddr;
use std::time::Duration;
use tracing::instrument;

use crate::http::{DEFAULT_ADDR, HttpServer};

use config::Config;
use signal::shutdown_channel;

pub use pal::{DaemonEvent, emit as pal_emit, take_events};

/// Loop task responsible for periodically flushing the write-ahead log.
///
/// This task demonstrates a simple system coroutine that would normally
/// interact with the `wal` crate. It records start and finish events in the
/// process activity log and sleeps briefly to simulate work.
#[instrument(skip(ctx))]
pub fn looptask_wal_flush(ctx: TaskContext) {
    pal_emit(DaemonEvent::WalFlushStart);
    tracing::info!("wal flush task started");
    ctx.syscall(SystemCall::Sleep(Duration::from_millis(10)));
    pal_emit(DaemonEvent::WalFlushFinish);
    tracing::info!("wal flush task finished");
}

/// Loop task responsible for pushing runtime metrics.
///
/// Similar to \[`looptask_wal_flush`\], this task emits PAL events so tests can
/// verify that it started. Real implementations would collect and export
/// metrics to a monitoring system.
#[instrument(skip(ctx))]
pub fn looptask_metrics(ctx: TaskContext) {
    pal_emit(DaemonEvent::MetricsStart);
    tracing::info!("metrics task started");
    ctx.syscall(SystemCall::Sleep(Duration::from_millis(10)));
    pal_emit(DaemonEvent::MetricsFinish);
    tracing::info!("metrics task finished");
}

/// Initialize the daemon and return a running instance.
#[instrument(skip(cfg))]
pub fn init(cfg: Config) -> anyhow::Result<Daemon> {
    tracing::info!("initializing daemon");
    Ok(Daemon { cfg })
}

/// Convenience wrapper to initialize and immediately run the daemon.
pub fn run(cfg: Config, dump_state: bool) -> anyhow::Result<()> {
    init(cfg)?.run(dump_state)
}

/// Drive the scheduler until a shutdown signal is received.
///
/// This loops over \[`Scheduler::run`\] to process any queued tasks and waits for
/// a termination notification from \[`signal::shutdown_channel`\]. When the
/// scheduler becomes idle it blocks on the receiver for a short interval so
/// that the loop does not busy spin.
#[instrument(skip(sched))]
fn run_blocking(sched: &mut Scheduler, dump: bool) -> anyhow::Result<()> {
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
    if dump {
        let state = sched.dump_state();
        tracing::info!(?state, "scheduler state");
    }
    Ok(())
}

/// Daemon state returned from \[`init`\].
pub struct Daemon {
    cfg: Config,
}

impl Daemon {
    /// Run the daemon until a termination signal is delivered.
    #[instrument(skip(self))]
    pub fn run(self, dump_state: bool) -> anyhow::Result<()> {
        tracing::info!("daemon running");
        // Watch for config changes (stub).
        let _watcher = signal::start_watcher(&self.cfg.config_path).ok();

        let addr: SocketAddr = DEFAULT_ADDR.parse().expect("valid addr");
        let http = HttpServer::start(addr)?;

        #[cfg(feature = "ipc")]
        let ipc = ipc::IpcService::start();
        #[cfg(feature = "grpc")]
        let grpc = grpc::GrpcService::start();
        #[cfg(feature = "a2a")]
        let a2a = a2a::A2aService::start();

        let mut sched = Scheduler::new();
        unsafe {
            sched.spawn_system(looptask_wal_flush);
            sched.spawn_system(looptask_metrics);
        }
        run_blocking(&mut sched, dump_state)?;

        http.shutdown();

        #[cfg(feature = "ipc")]
        ipc.shutdown();
        #[cfg(feature = "grpc")]
        grpc.shutdown();
        #[cfg(feature = "a2a")]
        a2a.shutdown();

        pal::emit(DaemonEvent::ShutdownComplete);
        tracing::info!("daemon shutdown complete");
        Ok(())
    }
}
