mod cache;
mod config;
mod context;
mod entity;
mod progress;

use context::Context;
use entity::Docset;
use futures::{stream::FuturesUnordered, StreamExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let context = Context::new().await;
    update_all(&context).await?;
    Ok(())
}

async fn update_all(context: &Context) -> anyhow::Result<()> {
    let docsets = Docset::try_to_fetch_docsets(context).await?;
    let pb = context.bar.add_root();
    pb.update_style(
        indicatif::ProgressStyle::default_bar()
            .template(
                "{prefix}{spinner:.green} [{bar:40.cyan/blue}] {human_pos}/{human_len} {wide_msg}",
            )
            .unwrap(),
    );

    let mut items = docsets.iter();
    let mut futures = FuturesUnordered::new();

    loop {
        while futures.len() < 5 {
            let docset = match items.next() {
                Some(docset) => docset,
                None => break,
            };
            let fut = docset.update_all(context, &pb);
            futures.push(fut);
        }
        if futures.is_empty() {
            break;
        }
        if let Some(res) = futures.next().await {
            res?;
            pb.inc(1);
        }
    }

    pb.finish("All docsets updated");
    Ok(())
}
