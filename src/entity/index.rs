use serde::{Deserialize, Serialize};

use crate::context::Context;

use super::Docset;

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexEntry {
    pub name: String,
    pub path: String,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexType {
    pub name: String,
    pub count: u32,
    pub slug: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    pub entries: Vec<IndexEntry>,
    pub types: Vec<IndexType>,
}

// impl Entry {
//     pub async fn try_to_update(context: &Context, docset: &Docset) -> anyhow::Result<Vec<Entry>> {
//         let filename = format!("{}/index.json", docset.base_directory());
//
//         if context.cache_file_exists(&filename) {
//             context.read_from_cache(&filename).await
//         } else {
//             let url = format!(
//                 "https://documents.devdocs.io/{}/index.json?{}",
//                 docset.slug, docset.mtime
//             );
//             context.download_file(&filename, url).await
//         }
//     }
// }
