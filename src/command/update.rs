use std::{collections::HashSet, path::PathBuf};

use clap::Args;
use futures::{stream::FuturesUnordered, StreamExt};

use crate::{config::Config, context::Context, entity::Docset};

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
    /// Suppress all progress bars.
    #[arg(long, default_value = "false")]
    no_progress: bool,
    /// Specify the temparory directory to store the downloaded files.
    #[arg(long)]
    cache_dir: Option<PathBuf>,
    /// The number of concurrent downloads.
    #[arg(short, long, default_value = "5")]
    limit: usize,
    slugs: Vec<String>,
}

impl From<UpdateArgs> for Config {
    fn from(args: UpdateArgs) -> Self {
        Self {
            cache_dir: args.cache_dir,
            progress: if args.no_progress { Some(false) } else { None },
            update_interval: None,
            force: if args.force { Some(true) } else { None },
        }
    }
}

#[async_trait::async_trait]
impl Command for UpdateArgs {
    async fn run(&self, context: &mut Context) -> anyhow::Result<()> {
        context.config = context.config.clone().extends(self.clone().into());
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
            while futures.len() < 5 {
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
