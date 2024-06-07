mod doc;
mod config;

use std::collections::HashSet;

use crate::doc::Docset;
use serde_json::Value;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let resp = reqwest::get("https://devdocs.io/docs.json")
        .await?
        .json::<Vec<Docset>>()
        .await?;

    let types = resp
        .iter()
        .map(|entry| entry.r#type.clone())
        .collect::<HashSet<String>>();

    Ok(())
}
