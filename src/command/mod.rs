use crate::context::Context;

pub mod update;

#[async_trait::async_trait]
pub trait Command {
    /// Run the command.
    async fn run(&self, context: &mut Context) -> anyhow::Result<()>;
}
