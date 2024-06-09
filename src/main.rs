mod cache;
mod config;
mod context;
mod entity;

use context::Context;
use entity::get_docsets;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let context = Context::new();

    let docsets = get_docsets(&context).await?;

    // println!("Found {} docsets", docsets.len());
    // println!("Docsets: {:#?}", docsets);

    Ok(())
}
