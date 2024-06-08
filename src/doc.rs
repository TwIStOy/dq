use std::{collections::HashMap, env, ffi::CString, os::fd::FromRawFd};

use anyhow::bail;
use bytes::BytesMut;
use futures::StreamExt;
use indicatif::ProgressBar;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

use crate::config::Config;

const DEVDOCS_META_URL: &str = "https://devdocs.io/docs.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Docset {
    pub name: String,
    pub slug: String,
    pub r#type: String,
    pub links: Option<HashMap<String, String>>,
    version: Option<String>,
    release: Option<String>,
    mtime: i64,
    db_size: i64,
}

pub async fn get_docsets(opts: &Config, client: &Client) -> anyhow::Result<Vec<Docset>> {
    let response = client.get(DEVDOCS_META_URL).send().await?;
    let total_size = response.content_length();

    let pb = if opts.progress() {
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
            pb.set_message(format!("Fetch docsets, {} bytes", downloaded))
        }
    }

    if let Some(pb) = pb.as_ref() {
        pb.finish_with_message(format!("Fetch docsets done, {} bytes", downloaded))
    }

    let docsets: Vec<Docset> = serde_json::from_slice(&payload)?;
    write_docset_to_file(opts, &docsets).await?;

    Ok(docsets)
}

async fn write_docset_to_file(opts: &Config, docset: &Vec<Docset>) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(opts.cache_dir()).await?;
    let cache_path = opts.cache_dir().join("docsets.json");
    let tmp_dir = env::temp_dir();
    let ptr = match CString::new(format!("{}/dq-cache-XXXXXX.json", tmp_dir.display())) {
        Ok(p) => p.into_raw(),
        Err(e) => bail!(e),
    };

    let fd = unsafe { libc::mkstemps(ptr, 5) };
    let path = match unsafe { CString::from_raw(ptr) }.into_string() {
        Ok(s) => s,
        Err(e) => bail!(e),
    };

    if fd < 0 {
        bail!(std::io::Error::last_os_error());
    }

    let mut tmpfile = unsafe { tokio::fs::File::from_raw_fd(fd) };
    tmpfile
        .write_all(serde_json::to_string_pretty(&docset).unwrap().as_bytes())
        .await?;

    tokio::fs::rename(path, cache_path).await?;
    Ok(())
}
