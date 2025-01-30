use anyhow::Result;
use clap::Parser;
use dotenv::dotenv;
use tracing_subscriber::EnvFilter;

mod cli;
mod prompt;
mod doc;
mod state;
mod build;
mod project_generator;
mod inference;
mod tools;

use build_system::cli::Cli;
use build_system::cli::run_cli;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Parse command-line arguments
    let cli = Cli::parse();

    // Run the CLI
    run_cli(cli).await
}
