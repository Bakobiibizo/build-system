use anyhow::Result;
use clap::{Parser, Subcommand};
use crate::tools;

#[derive(Parser, Debug)]
#[command(name = "tools")]
#[command(about = "Execute build system tools")]
pub struct ToolsCli {
    #[command(subcommand)]
    command: ToolCommands,
}

#[derive(Subcommand, Debug)]
enum ToolCommands {
    /// List all available tools
    List,
    
    /// Show detailed information about a specific tool
    Info {
        /// Name of the tool to show info for
        name: String,
    },
    
    /// Execute the build tool
    Build {
        /// Build command to execute (build, test, dev, clean)
        #[arg(short, long)]
        command: String,
        
        /// Working directory for the build command
        #[arg(short, long)]
        dir: String,
    },

    /// Generate a new project
    Project {
        /// Project name (in kebab-case)
        #[arg(long)]
        name: String,
        
        /// Programming language for the project
        #[arg(long)]
        language: String,
        
        /// Optional project description
        #[arg(long)]
        description: Option<String>,
    },
}

impl ToolsCli {
    pub async fn execute(&self) -> Result<()> {
        match &self.command {
            ToolCommands::List => {
                println!("Available tools:");
                println!("  - project: Generate a new project");
                println!("  - build: Execute build commands");
                Ok(())
            },
            ToolCommands::Info { name } => {
                match name.as_str() {
                    "project" => {
                        println!("project - Generate a new project");
                        println!("\nUsage: build-system tools project --name <name> --language <language>");
                        println!("\nArguments:");
                        println!("  --name        Project name (in kebab-case)");
                        println!("  --language    Programming language for the project");
                        println!("  --description Optional project description");
                    },
                    "build" => {
                        println!("build - Execute build commands");
                        println!("\nUsage: build-system tools build --command <command> --dir <directory>");
                        println!("\nArguments:");
                        println!("  --command    Build command to execute (build, test, dev, clean)");
                        println!("  --dir        Working directory for the build command");
                    },
                    _ => println!("Unknown tool: {}", name),
                }
                Ok(())
            },
            ToolCommands::Build { command, dir } => {
                println!("Executing build command: {} in directory: {}", command, dir);
                Ok(())
            },
            ToolCommands::Project { name, language, description } => {
                // Pass the arguments directly to the project tool
                let args = vec![
                    "project".to_string(),
                    "--name".to_string(),
                    name.clone(),
                    "--language".to_string(),
                    language.clone(),
                ];
                tools::run_tool("project", args).await
            }
        }
    }
}
