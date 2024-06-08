use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    /// The directory where the cache is stored.
    cache_dir: Option<PathBuf>,
    /// Whether to show progress bars.
    progress: Option<bool>,
}

static DEFAULT_CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(default_cache_dir);
fn default_cache_dir() -> PathBuf {
    let base_dir = xdg::BaseDirectories::with_prefix("dq").unwrap();
    base_dir.get_cache_home()
}

impl Config {
    pub fn cache_dir(&self) -> &Path {
        self.cache_dir
            .as_deref()
            .unwrap_or(DEFAULT_CACHE_DIR.as_path())
    }

    pub fn progress(&self) -> bool {
        self.progress.unwrap_or(true)
    }

    pub fn extends(self, other: Config) -> Self {
        Self {
            cache_dir: other.cache_dir.or(self.cache_dir),
            progress: other.progress.or(self.progress),
        }
    }

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
