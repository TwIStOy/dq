use std::collections::HashSet;

use clap::Args;
use futures::{stream::FuturesUnordered, StreamExt};

use crate::{context::Context, entity::Docset};

use super::Command;

#[derive(Args, Clone, Debug)]
pub struct UpdateArgs {
    /// Usually, the update command will not update the docsets meta if it has been updated
    /// recently. Also, the update command will not update these "index" or "db" if they are
    /// already exist.
    ///
    /// This flag disables these checks and forces the update command to update everything.
    #[arg(short, long, default_value = "false")]
    force: bool,
    /// Update all docsets instead of the specified ones.
    #[arg(long, default_value = "false")]
    all: bool,
    /// If specified, only update the specified docsets.
    slugs: Vec<String>,
}

#[async_trait::async_trait]
impl Command for UpdateArgs {
    async fn run(&self, context: &mut Context) -> anyhow::Result<()> {
        let docsets = Docset::try_to_fetch_docsets(context).await?;
        let pb = context.bar.add_root();
        pb.update_style(
        indicatif::ProgressStyle::default_bar()
            .template(
                "{prefix}{spinner:.green} [{bar:40.cyan/blue}] {human_pos}/{human_len} {wide_msg}",
            )
            .unwrap(),
        );

        let slugs = self.slugs.iter().collect::<HashSet<_>>();
        let filter = |docset: &&Docset| {
            if self.all {
                true
            } else {
                slugs.contains(&docset.slug)
            }
        };

        let items = docsets.iter().filter(filter).collect::<Vec<_>>();
        let mut iter = items.iter();

        if items.is_empty() {
            pb.finish("No docsets to update");
            return Ok(());
        }

        pb.set_length(items.len() as u64);

        let mut futures = FuturesUnordered::new();

        loop {
            while futures.len() < context.config.limit.unwrap_or(5) {
                let docset = match iter.next() {
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
}
