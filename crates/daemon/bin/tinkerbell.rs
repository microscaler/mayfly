use daemon::config::Config;

fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration
    let path = std::env::args().nth(1).unwrap_or_else(|| "config.toml".into());
    let cfg = Config::load(&path)?;

    // Start daemon
    daemon::run(cfg)
}
