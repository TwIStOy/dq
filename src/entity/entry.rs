use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::context::Context;

use super::Docset;

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    name: String,
    path: String,
    r#type: String,
}

fn format_index_url(docset: &Docset, filename: &str) -> String {
    format!(
        "https://documents.devdocs.io/{}/{}?{}",
        docset.slug, filename, docset.mtime
    )
}

pub async fn get_index_entries(context: &Context, docset: &Docset) -> anyhow::Result<Vec<Entry>> {
    context.download_file(PathBuf::from(""), format_index_url(docset, "index.json")).await
}
