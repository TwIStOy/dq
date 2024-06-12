use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Config {
    /// The directory where the cache is stored.
    pub cache_dir: Option<PathBuf>,
    /// Whether to show progress bars.
    pub progress: Option<bool>,
    /// The interval to update the cache. in seconds.
    pub update_interval: Option<u64>,
    /// Whether to force update the cache.
    #[serde(skip)]
    pub force: Option<bool>,
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

    pub fn update_interval(&self) -> u64 {
        self.update_interval.unwrap_or(60 * 60 * 24)
    }

    pub fn force(&self) -> bool {
        self.force.unwrap_or(false)
    }

    pub fn extends(self, other: Config) -> Self {
        Self {
            cache_dir: other.cache_dir.or(self.cache_dir),
            progress: other.progress.or(self.progress),
            update_interval: other.update_interval.or(self.update_interval),
            force: other.force.or(self.force),
        }
    }

    pub fn new_from_file() -> Self {
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
