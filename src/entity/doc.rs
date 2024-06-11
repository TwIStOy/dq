use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

use crate::context::Context;

use super::Entry;

const DEVDOCS_META_URL: &str = "https://devdocs.io/docs.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Docset {
    pub name: String,
    pub slug: String,
    pub r#type: String,
    pub links: Option<HashMap<String, String>>,
    pub version: Option<String>,
    pub release: Option<String>,
    pub mtime: i64,
    pub db_size: i64,

    #[serde(skip)]
    entries: OnceCell<Vec<Entry>>,
}

impl Docset {
    /// Try to update all docsets if outdated, then return them.
    pub async fn try_to_update_all(context: &Context) -> anyhow::Result<Vec<Docset>> {
        if context.cache_file_exists("docsets.json") && !context.caches.should_refresh_cache() {
            return context.read_from_cache("docsets.json").await;
        }

        let response = context.client.get(DEVDOCS_META_URL).send().await?;
        let pb = context.bar.add_root();

        return context
            .download_file("docsets.json", DEVDOCS_META_URL)
            .await;
    }

    pub fn base_directory(&self) -> String {
        format!("{}/{}", self.slug, self.mtime)
    }

    pub async fn get_entries(&self, context: &Context) -> anyhow::Result<&Vec<Entry>> {
        Ok(self
            .entries
            .get_or_init(|| async { Entry::try_to_update(context, self).await.unwrap() })
            .await)
    }
}
