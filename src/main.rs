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
    
    // Initialize PromptManager
    let template_dir = "./src/prompt/templates".to_string();
    let prompt_manager = prompt::PromptManager::new(template_dir).await;
    
    // Example project generation
    let sample_request = "Create a task management web application with user authentication";
    let project_config = prompt_manager.generate_project(sample_request).await?;
    
    info!("Generated project: {}", project_config.project_name);
    
    Ok(())
}
