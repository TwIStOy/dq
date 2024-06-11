use std::{env, ffi::CString, os::fd::FromRawFd, path::Path, time::Duration};

use anyhow::bail;
use bytes::{Bytes, BytesMut};
use futures::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget};
use reqwest::{Client, IntoUrl, Response};
use serde::{de::DeserializeOwned, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{cache::CachesManager, config::Config, progress::ProgressBarManager};

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
        let config = Config::new();
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

    pub async fn download_file<P, T, S>(&self, filename: P, url: S) -> anyhow::Result<T>
    where
        P: AsRef<Path>,
        T: Serialize + DeserializeOwned,
        S: IntoUrl,
    {
        let response = self.client.get(url).send().await?;
        let payload = self.download_with_progress(response).await?;
        let value: T = serde_json::from_slice(&payload)?;
        self.write_to_cache(filename, &value).await?;
        Ok(value)
    }

    pub async fn download_with_progress(&self, response: Response) -> anyhow::Result<Bytes> {
        let total_size = response.content_length();

        let pb = if let Some(total_size) = total_size {
            let bar = ProgressBar::new(total_size);
            bar.enable_steady_tick(Duration::from_millis(50));
            // bar.set_style(
            //     indicatif::ProgressStyle::default_bar()
            //         .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            //         .progress_chars("#>-"),
            // );
            bar
        } else {
            let bar = ProgressBar::new_spinner();
            bar
        };

        let pb = if self.config.progress() {
            let pb = if let Some(total_size) = total_size {
                ProgressBar::new(total_size)
            } else {
                ProgressBar::new_spinner()
            };
            Some(pb)
        } else {
            None
        };

        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();
        let mut payload = BytesMut::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            downloaded += chunk.len() as u64;
            payload.extend_from_slice(&chunk);
            if let Some(pb) = pb.as_ref() {
                pb.set_position(downloaded);
                pb.set_message(format!("Fetch docsets, {} bytes", downloaded))
            }
        }

        if let Some(pb) = pb.as_ref() {
            pb.finish_with_message(format!("Fetch docsets done, {} bytes", downloaded))
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

    pub async fn read_from_cache<T, F>(&self, filename: F) -> anyhow::Result<T>
    where
        T: DeserializeOwned,
        F: AsRef<Path>,
    {
        let cache_path = self.config.cache_dir().join(filename);
        let mut file = tokio::fs::File::open(cache_path).await?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await?;
        let value = serde_json::from_slice(&buf)?;
        Ok(value)
    }

    pub async fn write_to_cache<T, F>(&self, filename: F, value: &T) -> anyhow::Result<()>
    where
        T: Serialize,
        F: AsRef<Path>,
    {
        tokio::fs::create_dir_all(self.config.cache_dir()).await?;

        let cache_path = self.config.cache_dir().join(filename);
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
            bail!(std::io::Error::last_os_error());
        }

        let mut tmpfile = unsafe { tokio::fs::File::from_raw_fd(fd) };
        let value = serde_json::to_string_pretty(value).unwrap();
        tmpfile.write_all(value.as_bytes()).await?;

        tokio::fs::rename(path, cache_path).await?;

        Ok(())
    }
}
