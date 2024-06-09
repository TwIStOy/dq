use std::{path::PathBuf, sync::Arc, time::Duration};

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
        let update_interval = Duration::from_secs(opts.update_interval().unwrap_or(3600));
        let force = opts.force().unwrap_or(false);

        Self {
            root: root.to_path_buf(),
            last_modified,
        }
    }

    pub fn should_refresh_cache(&self) -> bool {
        false
    }
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct CacheItem {
//     last_modified: i64,
// }
//
// #[derive(Debug, Serialize, Deserialize)]
// pub struct CachesMeta {
//     last_updated: i64,
// }
//
// #[derive(Debug, Serialize, Deserialize)]
// pub struct Caches {
//     root: PathBuf,
//     cache_meta: HashMap<PathBuf, CacheItem>,
// }
//
// impl Caches {
//     pub fn new(opts: &Config) -> Self {
//         Self {
//             root: opts.cache_dir().to_path_buf(),
//             caches: HashMap::new(),
//         }
//     }
// }
