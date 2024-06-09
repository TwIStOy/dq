use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::context::Context;

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

pub async fn get_docsets(context: &Context) -> anyhow::Result<Vec<Docset>> {
    context
        .download_file("docsets.json", DEVDOCS_META_URL)
        .await
}
