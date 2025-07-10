use daemon::config::Config;
use tracing_subscriber::{EnvFilter, fmt};

/// Main entry point for the `tinkerbell` daemon binary.
///
/// This function initializes logging, loads configuration from disk,
/// and delegates execution to [`daemon::run`].
#[tracing::instrument]
fn main() -> anyhow::Result<()> {
    // Initialize logging using RUST_LOG if set
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).init();

    // Load configuration
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config.toml".into());
    tracing::debug!(%path, "loading configuration");
    let cfg = Config::load(&path)?;

    // Run the daemon
    daemon::run(cfg)
}
