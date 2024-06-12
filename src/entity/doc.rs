use std::{collections::HashMap, path::Path, sync::Arc};

use futures::{stream::FuturesUnordered, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

use crate::{context::Context, progress::ProgressBar};

use super::{Index, IndexEntry};

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
}

impl Docset {
    /// Try to update all docsets if outdated, then return them.
    pub async fn try_to_fetch_docsets(context: &Context) -> anyhow::Result<Vec<Docset>> {
        if context.cache_file_exists("docsets.json") && !context.caches.should_refresh_cache() {
            return context.read_from_cache("docsets.json").await;
        }
        let pb = context.bar.add_root();
        let ret = context
            .download_file("docsets.json", DEVDOCS_META_URL, &pb)
            .await?;
        pb.finish("docsets.json downloaded");
        Ok(ret)
    }

    pub fn base_directory(&self) -> String {
        format!("{}/{}", self.slug, self.mtime)
    }

    async fn fetch_index(
        &self,
        context: &Context,
        parent: &Arc<ProgressBar>,
    ) -> anyhow::Result<Index> {
        let url = format!(
            "https://documents.devdocs.io/{}/index.json?{}",
            self.slug, self.mtime
        );
        let filename = self.base_directory() + "/index.json";
        let pb = context.bar.add_child_with_total(parent, None);
        let index: Index = context.download_file(filename, url, &pb).await?;
        pb.finish(format!(
            "{} index downloaded, got {} entries",
            self.name,
            index.entries.len()
        ));
        context.bar.remove_bar(&pb);
        Ok(index)
    }

    async fn fetch_db(
        &self,
        context: &Context,
        parent: &Arc<ProgressBar>,
    ) -> anyhow::Result<HashMap<String, String>> {
        let url = format!(
            "https://documents.devdocs.io/{}/db.json?{}",
            self.slug, self.mtime
        );
        let filename = self.base_directory() + "/db.json";
        let pb = context.bar.add_child_with_total(parent, None);
        let db = context.download_file(filename, url, &pb).await?;
        pb.finish(format!("{} db downloaded", self.name));
        context.bar.remove_bar(&pb);
        Ok(db)
    }

    async fn write_page(path: impl AsRef<Path>, data: &str) -> anyhow::Result<()> {
        if let Some(parent) = path.as_ref().parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(path, data.as_bytes()).await?;
        Ok(())
    }

    async fn unpack_db(
        &self,
        context: &Context,
        parent: &Arc<ProgressBar>,
        db: &HashMap<String, String>,
    ) -> anyhow::Result<()> {
        let pb = context
            .bar
            .add_child_with_total(parent, Some(db.len() as u64));
        let db_base_directory = context
            .config
            .cache_dir()
            .join(self.base_directory())
            .join("db");

        let mut items = db.iter();
        let mut futures = FuturesUnordered::new();

        loop {
            while futures.len() < 4 {
                if let Some((path, content)) = items.next() {
                    let filename = db_base_directory.join(path).join("_index");
                    let fut = Self::write_page(filename, content);
                    futures.push(fut);
                } else {
                    break;
                }
            }
            if futures.is_empty() {
                break;
            }
            if let Some(res) = futures.next().await {
                res?;
                pb.inc(1);
            }
        }

        pb.finish(format!("{} pages written", db.len()));
        context.bar.remove_bar(&pb);
        Ok(())
    }

    pub async fn update_all(
        &self,
        context: &Context,
        parent: &Arc<ProgressBar>,
    ) -> anyhow::Result<Index> {
        let pb = context.bar.add_msg(Some(parent));
        pb.set_message(format!("Updating {}", self.name));

        let (index, db) = tokio::join!(self.fetch_index(context, &pb), self.fetch_db(context, &pb));
        let index = index?;
        let db = db?;
        self.unpack_db(context, &pb, &db).await?;

        Ok(index)
    }
}
