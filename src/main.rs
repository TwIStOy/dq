mod cache;
mod command;
mod config;
mod context;
mod entity;
mod utils;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use command::Command as _;
use context::Context;

#[derive(Debug, Parser)]
#[command(name = "dq")]
#[command(about = "A fictional versioning CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Suppress all progress bars.
    #[arg(global = true, long, default_value = "false")]
    no_progress: bool,
    /// Specify the temparory directory to store the downloaded files.
    #[arg(global = true, long)]
    cache_dir: Option<PathBuf>,
    /// The number of concurrent downloads.
    #[arg(global = true, short, long, default_value = "5")]
    limit: Option<usize>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Update the docsets.
    Update(command::update::UpdateArgs),
    /// Search in the docsets.
    Search(command::search::SearchArgs),
    /// Display a doc page.
    Cat(command::cat::CatArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut context = Context::new().await;

    let cli = Cli::parse();
    cli.update_config(&mut context);

    match cli.command {
        Commands::Update(args) => args.run(&mut context).await?,
        Commands::Search(args) => args.run(&mut context).await?,
        Commands::Cat(args) => args.run(&mut context).await?,
    }

    Ok(())
}

impl Cli {
    fn update_config(&self, context: &mut Context) {
        if let Some(cache_dir) = &self.cache_dir {
            context.config.cache_dir = Some(cache_dir.clone());
        }
        if self.no_progress {
            context.config.progress = Some(false);
        }
        if let Some(limit) = self.limit {
            context.config.limit = Some(limit);
        }
    }
}
