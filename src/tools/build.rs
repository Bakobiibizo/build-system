use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::tools::{Tool, ExecutableTool};
use async_trait::async_trait;
use tokio::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildTool {
    name: String,
    description: String,
    parameters: BuildToolParameters,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildToolParameters {
    #[serde(rename = "type")]
    type_: String,
    properties: BuildToolProperties,
    required: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildToolProperties {
    command: CommandProperty,
    working_directory: WorkingDirProperty,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandProperty {
    #[serde(rename = "type")]
    type_: String,
    description: String,
    #[serde(rename = "enum")]
    allowed_values: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkingDirProperty {
    #[serde(rename = "type")]
    type_: String,
    description: String,
}

impl Default for BuildTool {
    fn default() -> Self {
        Self {
            name: "build".to_string(),
            description: "Execute build commands for the project".to_string(),
            parameters: BuildToolParameters {
                type_: "object".to_string(),
                properties: BuildToolProperties {
                    command: CommandProperty {
                        type_: "string".to_string(),
                        description: "The build command to execute".to_string(),
                        allowed_values: vec![
                            "build".to_string(),
                            "test".to_string(),
                            "dev".to_string(),
                            "clean".to_string(),
                        ],
                    },
                    working_directory: WorkingDirProperty {
                        type_: "string".to_string(),
                        description: "The directory to execute the build command in".to_string(),
                    },
                },
                required: vec!["command".to_string(), "working_directory".to_string()],
            },
        }
    }
}

#[async_trait]
impl ExecutableTool for BuildTool {
    async fn execute(&self, arguments: &str) -> Result<String, String> {
        let args: serde_json::Value = serde_json::from_str(arguments)
            .map_err(|e| format!("Failed to parse arguments: {}", e))?;
        
        let command = args["command"].as_str()
            .ok_or("Missing command parameter")?;
        let working_dir = args["working_directory"].as_str()
            .ok_or("Missing working_directory parameter")?;

        // Execute the appropriate build command based on the project type
        match command {
            "build" => {
                // Check for setup.py or requirements.txt
                if std::path::Path::new(&format!("{}/setup.py", working_dir)).exists() {
                    let output = Command::new("python")
                        .args(&["setup.py", "build"])
                        .current_dir(working_dir)
                        .output()
                        .await
                        .map_err(|e| format!("Failed to execute build command: {}", e))?;
                    if output.status.success() {
                        Ok(String::from_utf8_lossy(&output.stdout).to_string())
                    } else {
                        Err(String::from_utf8_lossy(&output.stderr).to_string())
                    }
                } else {
                    let output = Command::new("pip")
                        .args(&["install", "-r", "requirements.txt"])
                        .current_dir(working_dir)
                        .output()
                        .await
                        .map_err(|e| format!("Failed to execute build command: {}", e))?;
                    if output.status.success() {
                        Ok(String::from_utf8_lossy(&output.stdout).to_string())
                    } else {
                        Err(String::from_utf8_lossy(&output.stderr).to_string())
                    }
                }
            }
            "test" => {
                let output = Command::new("python")
                    .args(&["-m", "pytest"])
                    .current_dir(working_dir)
                    .output()
                    .await
                    .map_err(|e| format!("Failed to execute test command: {}", e))?;
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    Err(String::from_utf8_lossy(&output.stderr).to_string())
                }
            }
            "dev" => {
                let output = Command::new("python")
                    .args(&["-m", "flask", "run", "--debug"])
                    .current_dir(working_dir)
                    .output()
                    .await
                    .map_err(|e| format!("Failed to execute dev command: {}", e))?;
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    Err(String::from_utf8_lossy(&output.stderr).to_string())
                }
            }
            "clean" => {
                // Remove build artifacts
                let _ = std::fs::remove_dir_all(format!("{}/build", working_dir));
                let _ = std::fs::remove_dir_all(format!("{}/__pycache__", working_dir));
                let _ = std::fs::remove_dir_all(format!("{}/.pytest_cache", working_dir));
                Ok("Clean completed successfully".to_string())
            }
            _ => Err(format!("Unknown command: {}", command)),
        }
    }

    fn get_tool_definition(&self) -> Tool {
        Tool {
            name: self.name.clone(),
            description: self.description.clone(),
            parameters: serde_json::to_value(&self.parameters).unwrap(),
        }
    }

    fn get_short_description(&self) -> String {
        "Execute build commands (build, test, dev, clean) for Python projects".to_string()
    }

    fn get_long_description(&self) -> String {
        r#"This tool executes build-related commands for Python projects. Available commands:
        - build: Install dependencies and build the project
        - test: Run the test suite using pytest
        - dev: Start the development server in debug mode
        - clean: Remove build artifacts and cache directories
        
        The tool automatically detects the project structure and uses appropriate build commands.
        For pip-based projects, it uses requirements.txt.
        For setuptools projects, it uses setup.py.
        "#.to_string()
    }
}
