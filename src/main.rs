mod doc;

use crate::doc::DocsEntry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let resp = reqwest::get("https://devdocs.io/docs.json")
        .await?
        .json::<Vec<DocsEntry>>()
        .await?;
    println!("{resp:#?}");

    Ok(())
}
