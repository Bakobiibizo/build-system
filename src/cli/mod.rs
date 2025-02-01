use anyhow::Result;
use clap::{Parser, Subcommand};
use crate::inference::InferenceClient;

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
    /// Generate a new project
    Generate {
        /// Project name
        #[arg(short = 'n', long)]
        name: String,
        
        /// Programming language
        #[arg(short, long)]
        language: String,

        /// Project description
        #[arg(short = 'd', long)]
        description: Option<String>,
    },
    
    /// Execute build system tools
    Tools(ToolsCli),
}

impl Cli {
    pub async fn execute(&self) -> Result<()> {
        match &self.command {
            Commands::Generate { name, language, description } => {
                // Generate project configuration using AI
                let inference = InferenceClient::new()?;
                let prompt = format!(
                    "Create a {} project named '{}' with the following description: {}",
                    language, name, description.as_deref().unwrap_or("")
                );
                
                println!("Full Prompt for Project Config Generation:");
                println!("{}", include_str!("../prompt/templates/project_generation_prompt.md"));
                println!("\n## User Request\n{}", prompt);
                
                println!("\n## Generate Configuration Based on Request");
                let config_json = inference.generate_project_config(&prompt).await?;
                
                println!("Generated Project Config JSON:");
                println!("{}", config_json);
                
                // Parse the config and create the project
                use crate::project_generator::{parse_project_design, ProjectGenerator};
                let project_design = parse_project_design(&config_json)?;
                let generator = ProjectGenerator::new(project_design);
                generator.generate().await?;
                
                println!("\nProject generated successfully!");
                Ok(())
            },
            Commands::Tools(tools_cli) => tools_cli.execute().await,
        }
    }
}

pub async fn handle_cli_command(cli: Cli) -> Result<()> {
    cli.execute().await
}
