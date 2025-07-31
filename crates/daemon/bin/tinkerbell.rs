use clap::Parser;
use daemon::config::Config;
use tracing_subscriber::{EnvFilter, fmt};

/// Command line arguments for the `tinkerbell` daemon.
#[derive(Parser, Debug)]
#[command(name = "tinkerbell", about = "Run the Tiffany daemon")]
struct Cli {
    /// Path to the configuration file.
    #[arg(value_name = "PATH", default_value = "config.toml")]
    config: String,

    /// Number of worker threads to use.
    #[arg(long)]
    concurrency: Option<usize>,

    /// Scheduling quantum in milliseconds.
    #[arg(long)]
    quantum: Option<u64>,

    /// Increase logging verbosity. Repeat for more detail.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Dump scheduler state on shutdown.
    #[arg(long)]
    dump_state: bool,
}

/// Main entry point for the `tinkerbell` daemon binary.
///
/// Parses command line arguments, configures logging, loads the
/// configuration from disk, and delegates execution to \[`daemon::run`\].
#[tracing::instrument]
fn main() -> anyhow::Result<()> {
    // Parse CLI flags
    let args = Cli::parse();

    // Configure logging based on verbosity flag or RUST_LOG
    let filter = match args.verbose {
        0 => EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        1 => EnvFilter::new("debug"),
        _ => EnvFilter::new("trace"),
    };
    fmt().with_env_filter(filter).init();

    tracing::debug!(?args, "parsed CLI arguments");

    // Load configuration
    tracing::debug!(path = %args.config, "loading configuration");
    let cfg = Config::load(&args.config)?;

    // Run the daemon
    daemon::run(cfg, args.dump_state)
}
