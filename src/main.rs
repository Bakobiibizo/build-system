use anyhow::Result;
use clap::Parser;
use dotenv::dotenv;
use tracing_subscriber::EnvFilter;

use build_system::cli::Cli;
mod prompt;
mod inference;
mod build;
mod state;
mod project_generator;
mod tools;

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
    Cli::run(cli).await
}
