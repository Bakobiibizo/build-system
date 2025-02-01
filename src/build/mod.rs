use std::path::PathBuf;
use std::fs::{self, File};
use std::io::Write;
use tokio::process::Command;
use serde_json::Value;
use anyhow::{Context, Result};
use jsonschema::JSONSchema;

use crate::state::types::{TaskId, TaskState, TaskStatus};
use crate::state::StateManager;

pub mod error;
pub use error::BuildError;

#[derive(Debug, Clone)]
pub struct BuildManager {
    pub state_manager: StateManager,
    working_dir: PathBuf,
}

impl BuildManager {
    pub fn new(state_manager: StateManager, working_dir: PathBuf) -> Self {
        Self { 
            state_manager, 
            working_dir 
        }
    }

    /// Validate JSON against a given schema
    pub fn validate_json(schema: &Value, data: &Value) -> Result<()> {
        // Create a 'static reference by leaking the schema
        let schema_static = Box::leak(Box::new(schema.clone()));
        let compiled_schema = JSONSchema::compile(schema_static)
            .context("Failed to compile JSON schema")?;

        if let Err(errors) = compiled_schema.validate(data) {
            let error_messages: Vec<String> = errors
                .map(|error| error.to_string())
                .collect();
            anyhow::bail!("JSON validation failed: {}", error_messages.join(", "))
        }

        Ok(())
    }

    // New method to scaffold a project from JSON configuration
    pub fn scaffold_project(&self, project_config: &str) -> Result<PathBuf> {
        // Parse the JSON configuration
        let config: Value = serde_json::from_str(project_config)
            .context("Failed to parse project configuration")?;

        // Extract project name
        let project_name = config["project_name"].as_str()
            .unwrap_or("unnamed_project")
            .to_string();

        // Create unique project directory
        let project_dir = self.working_dir.join(format!("{}_{}",
            project_name, 
            std::process::id()  // Add process ID to ensure uniqueness
        ));
        fs::create_dir_all(&project_dir)?;

        // Create directory structure
        self.create_directory_structure(&project_dir, &config)?;

        // Create initialization files
        self.create_initialization_files(&project_dir, &config)?;

        // Create configuration files
        self.create_config_files(&project_dir, &config)?;

        // Create documentation
        self.create_documentation(&project_dir, &config)?;

        Ok(project_dir)
    }

    fn create_directory_structure(&self, project_dir: &PathBuf, config: &Value) -> Result<()> {
        // Ensure the base directories are created
        let base_dirs = vec!["src", "tests", "migrations", "config"];
        for dir in base_dirs {
            fs::create_dir_all(project_dir.join(dir))?;
        }

        // Create subdirectories and files based on the directory_structure
        if let Some(dir_structure) = config["directory_structure"].as_object() {
            for (base_dir, entries) in dir_structure {
                let base_path = project_dir.join(base_dir);
                
                // Create base directory if it doesn't exist
                fs::create_dir_all(&base_path)?;

                // Create subdirectories and files
                if let Some(dir_list) = entries.as_array() {
                    for entry in dir_list {
                        if let Some(entry_str) = entry.as_str() {
                            let entry_path = base_path.join(entry_str);
                            
                            // Check if it's a directory or a file
                            if entry_str.contains('/') {
                                // It's a subdirectory
                                fs::create_dir_all(&entry_path)?;
                            } else {
                                // It's a file
                                if let Some(parent) = entry_path.parent() {
                                    fs::create_dir_all(parent)?;
                                }
                                File::create(&entry_path)?;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn create_initialization_files(&self, project_dir: &PathBuf, config: &Value) -> Result<()> {
        // Determine main file based on language
        let main_file_path = match config["language"].as_str() {
            Some("Rust") => project_dir.join("src/main.rs"),
            Some("JavaScript") => project_dir.join("src/app.js"),
            Some("Python") => project_dir.join("src/main.py"),
            _ => project_dir.join("src/main"),
        };

        // Ensure parent directory exists
        if let Some(parent) = main_file_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        // Create main file with boilerplate content
        let main_content = match config["language"].as_str() {
            Some("Rust") => {
                if config["framework"].as_str() == Some("Rocket") {
                    r#"#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Welcome to TaskMaster!"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}"#
                } else {
                    "fn main() {\n    println!(\"Hello, TaskMaster!\");\n}"
                }
            },
            Some("JavaScript") => 
                "console.log('TaskMaster application started');",
            Some("Python") => 
                "def main():\n    print('TaskMaster application started')\n\nif __name__ == '__main__':\n    main()",
            _ => "// Main application entry point",
        };

        std::fs::write(&main_file_path, main_content)
            .with_context(|| format!("Failed to write main file: {}", main_file_path.display()))?;

        // Create configuration files
        if let Some(config_files) = config["directory_structure"]["config"].as_array() {
            for config_file in config_files {
                if let Some(filename) = config_file.as_str() {
                    let config_path = project_dir.join("config").join(filename);
                    
                    // Create default content based on filename
                    let config_content = match filename {
                        "database.toml" => r#"[database]
host = "localhost"
port = 5432
name = "taskmaster"
username = "taskmaster_user"
password = "changeme"
"#,
                        "jwt.toml" => r#"[jwt]
secret_key = "your_secret_key_here"
expiration_hours = 24
"#,
                        _ => "# Configuration file",
                    };

                    std::fs::write(&config_path, config_content)
                        .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;
                }
            }
        }

        Ok(())
    }

    fn create_config_files(&self, project_dir: &PathBuf, config: &Value) -> Result<()> {
        // Create Cargo.toml for Rust projects
        if config["language"].as_str() == Some("Rust") {
            let cargo_toml_path = project_dir.join("Cargo.toml");
            
            // Prepare dependencies
            let mut prod_deps = String::new();
            let mut dev_deps = String::new();

            if let Some(dependencies) = config["dependencies"].as_object() {
                if let Some(production) = dependencies.get("production").and_then(|d| d.as_object()) {
                    for (name, version) in production {
                        prod_deps.push_str(&format!("{} = \"{}\"\n", name, version.as_str().unwrap_or("latest")));
                    }
                }

                if let Some(development) = dependencies.get("development").and_then(|d| d.as_object()) {
                    for (name, version) in development {
                        dev_deps.push_str(&format!("{} = \"{}\"\n", name, version.as_str().unwrap_or("latest")));
                    }
                }
            }

            let cargo_toml_content = format!(
                r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
{}

[dev-dependencies]
{}"#, 
                config["project_name"].as_str().unwrap_or("taskmaster"),
                prod_deps,
                dev_deps
            );

            std::fs::write(&cargo_toml_path, cargo_toml_content)
                .with_context(|| format!("Failed to write Cargo.toml: {}", cargo_toml_path.display()))?;
        }

        Ok(())
    }

    fn create_documentation(&self, project_dir: &PathBuf, config: &Value) -> Result<()> {
        // Create README.md
        let readme_path = project_dir.join("README.md");
        let mut readme = File::create(&readme_path)?;

        // Write project overview
        let readme_content = format!(
            "# {}\n\n## Description\n{}\n\n## Technologies\n{:?}\n\n## Recommendations\n{:?}",
            config["project_name"].as_str().unwrap_or("Project"),
            config["description"].as_str().unwrap_or(""),
            config["technologies"].as_array().unwrap_or(&vec![]),
            config["recommendations"].as_array().unwrap_or(&vec![])
        );

        readme.write_all(readme_content.as_bytes())?;

        Ok(())
    }

    pub async fn execute_task(&self, task_id: &TaskId) -> Result<(), BuildError> {
        // Get task from state manager
        let task = self.state_manager.get_task(task_id).await
            .map_err(BuildError::StateError)?;

        // Execute task command
        self.execute_command(&task).await?;

        // Update task status to completed
        self.state_manager.update_task_status(task_id, TaskStatus::Completed).await
            .map_err(BuildError::StateError)?;

        Ok(())
    }

    async fn execute_command(&self, task: &TaskState) -> Result<(), BuildError> {
        let command = &task.metadata.name;
        let args: Vec<&str> = command.split_whitespace().collect();

        if args.is_empty() {
            return Err(BuildError::InvalidCommand("Empty command".to_string()));
        }

        let output = Command::new(args[0])
            .args(&args[1..])
            .current_dir(&self.working_dir)
            .output()
            .await?;

        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(BuildError::CommandFailed(error_message));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use chrono::Utc;

    #[tokio::test]
    async fn test_execute_task() -> Result<(), BuildError> {
        let state_manager = StateManager::new();
        let build_manager = BuildManager::new(state_manager.clone(), PathBuf::from("/tmp"));

        // Create a test task
        let task_id = TaskId::new("test-task");
        let task = TaskState {
            id: task_id.clone(),
            status: TaskStatus::Pending,
            metadata: crate::state::types::TaskMetadata {
                name: "echo test".to_string(),
                description: Some("A test task".to_string()),
                owner: "test".to_string(),
                dependencies: vec![],
                estimated_duration: std::time::Duration::from_secs(60),
                priority: 1,
                tags: vec!["test".to_string()],
                additional_info: std::collections::HashMap::new(),
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        state_manager.create_task(task).await.map_err(BuildError::StateError)?;
        build_manager.execute_task(&task_id).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_project_scaffolding() -> Result<()> {
        // Create a comprehensive test configuration
        let project_config = serde_json::json!({
            "project_name": "TaskMaster",
            "description": "Web-based task management application with user authentication",
            "project_type": "WebApplication",
            "language": "Rust",
            "framework": "Rocket",
            "technologies": [
                "Rust",
                "PostgreSQL",
                "JWT Authentication",
                "Docker"
            ],
            "dependencies": {
                "production": {
                    "rocket": "0.5.0-rc.2",
                    "diesel": "1.4.8",
                    "jsonwebtoken": "8.2.0"
                },
                "development": {
                    "cargo-watch": "8.4.0",
                    "rust-analyzer": "latest"
                }
            },
            "build_config": {
                "build_tool": "cargo",
                "scripts": {
                    "dev": "cargo watch -x run",
                    "build": "cargo build --release",
                    "test": "cargo test"
                }
            },
            "directory_structure": {
                "src": ["main.rs", "routes/mod.rs", "models/mod.rs", "auth/mod.rs"],
                "tests": ["integration/mod.rs", "unit/mod.rs"],
                "migrations": [],
                "config": ["database.toml", "jwt.toml"]
            },
            "initialization_commands": [
                "cargo new taskmaster",
                "cd taskmaster",
                "cargo add rocket diesel jsonwebtoken",
                "diesel setup"
            ],
            "recommendations": [
                "Implement role-based access control",
                "Use environment variable configuration",
                "Implement comprehensive logging"
            ]
        });

        // Create a state manager and build manager
        let state_manager = StateManager::new();
        let build_manager = BuildManager::new(
            state_manager, 
            PathBuf::from("./build")  // Change working directory to ./build
        );

        // Ensure the build directory exists
        std::fs::create_dir_all("./build")?;

        // Scaffold the project
        let project_dir = build_manager.scaffold_project(&project_config.to_string())?;

        // Verify project directory exists
        println!("Project directory: {}", project_dir.display());
        assert!(project_dir.exists(), "Project directory should exist");

        // Verify specific entries
        let entries = vec![
            project_dir.join("src"),
            project_dir.join("src/main.rs"),
            project_dir.join("src/routes/mod.rs"),
            project_dir.join("src/models/mod.rs"),
            project_dir.join("src/auth/mod.rs"),
            project_dir.join("tests"),
            project_dir.join("tests/integration/mod.rs"),
            project_dir.join("tests/unit/mod.rs"),
            project_dir.join("migrations"),
            project_dir.join("config"),
            project_dir.join("config/database.toml"),
            project_dir.join("config/jwt.toml"),
            project_dir.join("Cargo.toml")
        ];

        for entry in entries {
            println!("Checking entry: {}", entry.display());
            assert!(entry.exists(), "Entry {} should exist", entry.display());
        }

        Ok(())
    }
}
