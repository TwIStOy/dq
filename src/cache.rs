use std::{
    path::PathBuf,
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Debug)]
pub struct CachesManager {
    root: PathBuf,
    last_modified: Duration,
    update_interval: Duration,
    force: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheMeta {
    last_modified: u64,
}

impl CachesManager {
    pub async fn new(opts: &Config) -> Self {
        let root = opts.cache_dir();
        let metafile = root.join("meta.json");

        let last_modified = if metafile.exists() {
            let meta = std::fs::read_to_string(&metafile).unwrap();
            let meta: CacheMeta = serde_json::from_str(&meta).unwrap();
            meta.last_modified
        } else {
            0
        };

        let last_modified = std::time::Duration::from_secs(last_modified);
        let update_interval = Duration::from_secs(opts.update_interval());
        let force = opts.force();

        Self {
            root: root.to_path_buf(),
            last_modified,
            update_interval,
            force,
        }
    }

    pub fn should_refresh_cache(&self) -> bool {
        let last_modified = SystemTime::UNIX_EPOCH + self.last_modified;
        let duration = SystemTime::now().duration_since(last_modified).unwrap();
        self.force || duration > self.update_interval
    }

    pub async fn flush_meta(&mut self) -> anyhow::Result<()> {
        let last_modified = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let meta = CacheMeta {
            last_modified: last_modified.as_secs(),
        };
        self.last_modified = last_modified;

        let meta = serde_json::to_string(&meta).unwrap();
        let metafile = self.root.join("meta.json");
        tokio::fs::write(metafile, meta).await?;
        Ok(())
    }
}
