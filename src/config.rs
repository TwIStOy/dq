use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

pub static CONFIG: LazyLock<Config> = LazyLock::new(Config::new_from_file);

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub cache_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        let base_dir = xdg::BaseDirectories::with_prefix("dq").unwrap();
        Self {
            cache_dir: base_dir.get_cache_home().to_string_lossy().to_string(),
        }
    }
}

impl Config {
    fn new_from_file() -> Self {
        match Self::load_from_file() {
            Ok(config) => config,
            Err(_) => Self::default(),
        }
    }

    fn load_from_file() -> anyhow::Result<Self> {
        let base_dir = xdg::BaseDirectories::with_prefix("dq").unwrap();
        let config_file = base_dir.get_config_file("config.toml");
        if config_file.exists() {
            let config = std::fs::read_to_string(config_file)?;
            let config = toml::from_str(&config)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }
}
