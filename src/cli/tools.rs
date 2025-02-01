use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use crate::tools::{ToolRegistry, ToolCall};
use serde_json::json;

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
        #[arg(short = 'n', long)]
        name: String,
        
        /// Programming language for the project
        #[arg(short = 'l', long)]
        language: String,
        
        /// Optional project description
        #[arg(short = 'd', long)]
        description: Option<String>,
        
        /// Optional JSON string containing a complete project design
        #[arg(short = 's', long)]
        design: Option<String>,
    },
}

impl ToolsCli {
    pub async fn execute(&self) -> Result<()> {
        let registry = ToolRegistry::new();
        
        match &self.command {
            ToolCommands::List => {
                println!("Available tools:");
                for (name, desc) in registry.get_tool_descriptions() {
                    println!("  {}: {}", name, desc);
                }
                Ok(())
            },
            
            ToolCommands::Info { name } => {
                if let Some(desc) = registry.get_tool_long_description(name) {
                    println!("{}", desc);
                    Ok(())
                } else {
                    Err(anyhow!("Tool '{}' not found", name))
                }
            },
            
            ToolCommands::Build { command, dir } => {
                let tool_call = ToolCall {
                    name: "build".to_string(),
                    arguments: json!({
                        "command": command,
                        "dir": dir,
                    }).to_string(),
                };
                
                match registry.execute_tool(&tool_call).await {
                    Ok(result) => {
                        println!("{}", result.output);
                        Ok(())
                    },
                    Err(e) => Err(anyhow!("Build failed: {}", e)),
                }
            },

            ToolCommands::Project { name, language, description, design } => {
                let tool_call = ToolCall {
                    name: "project".to_string(),
                    arguments: json!({
                        "name": name,
                        "language": language,
                        "description": description,
                        "design": design,
                    }).to_string(),
                };
                
                match registry.execute_tool(&tool_call).await {
                    Ok(result) => {
                        println!("{}", result.output);
                        Ok(())
                    },
                    Err(e) => Err(anyhow!("Project generation failed: {}", e)),
                }
            },
        }
    }
}
