use std::{
    env,
    ffi::CString,
    os::fd::FromRawFd,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::bail;
use bytes::{Bytes, BytesMut};
use futures::StreamExt;
use reqwest::{Client, IntoUrl, Response};
use serde::{de::DeserializeOwned, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    cache::CachesManager,
    config::Config,
    utils::progress::{ProgressBar, ProgressBarManager},
};

#[derive(Debug)]
pub struct Context {
    /// The configuration.
    pub config: Config,
    /// The HTTP client.
    pub client: Client,
    /// The caches.
    pub caches: CachesManager,
    /// The progress bar.
    pub bar: ProgressBarManager,
}

impl Context {
    /// Create a new context.
    pub async fn new() -> Self {
        let config = Config::new_from_file();
        let client = Client::new();
        let caches = CachesManager::new(&config).await;
        let bar = ProgressBarManager::new(&config);

        Self {
            config,
            client,
            caches,
            bar,
        }
    }

    pub async fn download_file<T, P, S>(
        &self,
        filename: P,
        url: S,
        pb: &Arc<ProgressBar>,
        skip_if_exists: bool,
    ) -> anyhow::Result<T>
    where
        P: AsRef<Path>,
        T: Serialize + DeserializeOwned,
        S: IntoUrl,
    {
        pb.set_message(format!("Downloading {}", filename.as_ref().display()));
        if skip_if_exists && self.cache_file_exists(filename.as_ref()) {
            let value = self.read_from_cache(filename.as_ref()).await?;
            Ok(value)
        } else {
            let response = self.client.get(url).send().await?;
            let payload = self.download_with_progress(response, pb).await?;
            let value: T = serde_json::from_slice(&payload).map_err(|e| {
                anyhow::anyhow!("Parse {}, err: {}", filename.as_ref().to_string_lossy(), e)
            })?;
            self.write_to_cache(filename.as_ref(), &value)
                .await
                .map_err(|e| {
                    anyhow::anyhow!("Write {}, err: {}", filename.as_ref().to_string_lossy(), e)
                })?;
            Ok(value)
        }
    }

    pub async fn download_with_progress(
        &self,
        response: Response,
        pb: &Arc<ProgressBar>,
    ) -> anyhow::Result<Bytes> {
        let total_size = response.content_length();
        pb.update_template(total_size);

        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();
        let mut payload = BytesMut::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            downloaded += chunk.len() as u64;
            payload.extend_from_slice(&chunk);
            pb.set_position(downloaded);
        }

        Ok(payload.freeze())
    }

    pub fn cache_file_exists<F>(&self, filename: F) -> bool
    where
        F: AsRef<Path>,
    {
        let cache_path = self.config.cache_dir().join(filename);
        cache_path.exists()
    }

    pub fn build_cache_path<F>(&self, filename: F) -> PathBuf
    where
        F: AsRef<Path>,
    {
        self.config.cache_dir().join(filename)
    }

    pub async fn read_from_cache<T, F>(&self, filename: F) -> anyhow::Result<T>
    where
        T: DeserializeOwned,
        F: AsRef<Path>,
    {
        let filename = filename.as_ref();
        let cache_path = self.config.cache_dir().join(filename);
        let mut file = tokio::fs::File::open(cache_path).await?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await?;
        let value = serde_json::from_slice(&buf)
            .map_err(|e| anyhow::anyhow!("Parse {}, err: {}", filename.to_string_lossy(), e))?;
        Ok(value)
    }

    pub async fn write_to_cache<T, F>(&self, filename: F, value: &T) -> anyhow::Result<()>
    where
        T: Serialize,
        F: AsRef<Path>,
    {
        let cache_path = self.config.cache_dir().join(filename);
        if let Some(parent) = cache_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let tmp_dir = env::temp_dir();
        let ptr = match CString::new(format!("{}/dq-cache-XXXXXX.cache", tmp_dir.display())) {
            Ok(p) => p.into_raw(),
            Err(e) => bail!(e),
        };

        let fd = unsafe { libc::mkstemps(ptr, 6) };
        let path = match unsafe { CString::from_raw(ptr) }.into_string() {
            Ok(s) => s,
            Err(e) => bail!(e),
        };

        if fd < 0 {
            bail!(
                "Failed to create temporary file: {}",
                std::io::Error::last_os_error()
            );
        }

        let mut tmpfile = unsafe { tokio::fs::File::from_raw_fd(fd) };
        let value = serde_json::to_string_pretty(value).unwrap();
        tmpfile.write_all(value.as_bytes()).await?;

        tokio::fs::rename(path, cache_path).await?;

        Ok(())
    }
}
