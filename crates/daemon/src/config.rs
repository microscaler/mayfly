use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Daemon configuration loaded from a TOML file.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Path to watch for configuration changes.
    #[serde(default = "default_config_path")]
    pub config_path: String,
}

fn default_config_path() -> String {
    "config.toml".to_string()
}

impl Config {
    /// Load configuration from the given path.
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(&path)?;
        let mut cfg: Self = toml::from_str(&contents)?;
        cfg.config_path = path.as_ref().to_string_lossy().to_string();
        Ok(cfg)
    }
}
