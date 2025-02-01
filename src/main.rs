use anyhow::Result;
use build_system::cli::{Cli, handle_cli_command};
use clap::Parser;
use dotenv::dotenv;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    handle_cli_command(cli).await
}
