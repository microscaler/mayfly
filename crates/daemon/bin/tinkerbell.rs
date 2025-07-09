use daemon::config::Config;

/// Main entry point for the `tinkerbell` daemon binary.
///
/// This function initializes logging, loads configuration from disk,
/// and delegates execution to [`daemon::run`].
fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration
    let path = std::env::args().nth(1).unwrap_or_else(|| "config.toml".into());
    let cfg = Config::load(&path)?;

    // Run the daemon
    daemon::run(cfg)
}
