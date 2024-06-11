mod cache;
mod config;
mod context;
mod entity;
mod progress;

use context::Context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let context = Context::new().await;

    // println!("Found {} docsets", docsets.len());
    // println!("Docsets: {:#?}", docsets);

    Ok(())
}
