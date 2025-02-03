use anyhow::Result;
use clap::{Parser, Subcommand};

mod tools;
use tools::ToolsCli;

#[derive(Parser)]
#[command(name = "build-system")]
#[command(about = "AI-powered build system")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Execute build system tools
    Tools(ToolsCli),
}

pub async fn handle_cli_command(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Tools(tools) => tools.execute().await,
    }
}
