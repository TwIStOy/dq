mod config;
mod doc;

use config::Config;
use doc::get_docsets;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config: Config = Default::default();
    let client = reqwest::Client::new();

    let docsets = get_docsets(&config, &client).await?;

    // println!("Found {} docsets", docsets.len());
    // println!("Docsets: {:#?}", docsets);

    Ok(())
}
