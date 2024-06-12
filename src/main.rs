mod cache;
mod command;
mod config;
mod context;
mod entity;
mod utils;

use clap::{Parser, Subcommand};
use command::Command as _;
use context::Context;

#[derive(Debug, Parser)]
#[command(name = "dq")]
#[command(about = "A fictional versioning CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Update the docsets.
    Update(command::update::UpdateArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut context = Context::new().await;

    let cli = Cli::parse();

    match cli.command {
        Commands::Update(args) => args.run(&mut context).await?,
    }

    Ok(())
}
