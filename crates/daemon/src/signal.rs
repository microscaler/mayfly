use crossbeam::channel::{Receiver, bounded};
use signal_hook::consts::signal::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use std::path::Path;

/// Register signal handlers for `SIGINT` and `SIGTERM` and return a channel
/// that receives a single notification when either signal is delivered.
///
/// The channel is backed by a dedicated thread that waits on a
/// \[`signal_hook::iterator::Signals`\] instance. This keeps the signal handling
/// minimal and allows the rest of the daemon to poll the receiver without
/// blocking.
pub fn shutdown_channel() -> anyhow::Result<Receiver<()>> {
    let (tx, rx) = bounded(1);
    let mut signals = Signals::new([SIGINT, SIGTERM])?;
    std::thread::spawn(move || {
        if let Some(_sig) = signals.forever().next() {
            let _ = tx.send(());
        }
    });
    Ok(rx)
}

/// Start watching the given config path for file modifications.
///
/// The returned watcher is configured to emit events for the provided path
/// and will log any notifications. Hot-reload logic for the configuration is
/// not yet implemented and thus the watcher is primarily used in tests.
#[allow(dead_code)]
pub fn start_watcher<P: AsRef<Path>>(path: P) -> notify::Result<notify::RecommendedWatcher> {
    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

    let mut watcher = RecommendedWatcher::new(
        |res| {
            if let Ok(event) = res {
                tracing::debug!(?event, "config changed - hot reload not yet implemented");
            }
        },
        Config::default(),
    )?;
    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;
    Ok(watcher)
}
