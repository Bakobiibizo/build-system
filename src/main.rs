use anyhow::Result;
use clap::Parser;
use tracing::info;

mod state;
mod prompt;
mod build;
mod doc;
mod cli;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Parse command line arguments
    let args = Args::parse();
    
    info!("Build system starting...");
    
    // TODO: Initialize components and start build system
    
    Ok(())
}
