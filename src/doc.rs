use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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

pub async fn get_docsets() -> anyhow::Result<Vec<Docset>> {
    Ok(reqwest::get("https://devdocs.io/docs.json")
        .await?
        .json::<Vec<Docset>>()
        .await?)
}
