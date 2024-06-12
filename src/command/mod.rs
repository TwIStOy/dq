use crate::context::Context;

pub mod update;
pub mod search;
pub mod cat;

#[async_trait::async_trait]
pub trait Command {
    /// Run the command.
    async fn run(&self, context: &mut Context) -> anyhow::Result<()>;
}
