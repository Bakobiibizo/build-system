use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export ProjectType from prompt/mod.rs
pub use crate::prompt::ProjectType;

/// Represents a comprehensive project generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectGenerationConfig {
    /// The name of the project in kebab-case
    pub project_name: String,

    /// Project description
    pub description: String,

    /// Primary programming language
    pub language: String,

    /// Web framework or library
    pub framework: String,

    /// Type of project
    pub project_type: ProjectType,

    /// List of technologies used
    pub technologies: Vec<String>,

    /// Project components and their responsibilities
    pub components: HashMap<String, String>,

    /// Detailed directory structure
    pub directory_structure: HashMap<String, Vec<String>>,

    /// Production and development dependencies
    pub dependencies: DependencyConfig,

    /// Build and configuration details
    pub build_config: BuildConfig,

    /// Commands to initialize the project
    pub initialization_commands: Vec<String>,

    /// Additional recommendations
    pub recommendations: Vec<String>,
}

/// Represents different types of software projects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectType {
    WebApplication,
    CommandLineInterface,
    Library,
    MicroService,
    DesktopApplication,
    MobileApplication,
}

/// Configuration for project dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyConfig {
    /// Core production dependencies
    pub production: HashMap<String, String>,

    /// Development and testing dependencies
    pub development: HashMap<String, String>,
}

/// Build and configuration details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Build tool or system
    pub build_tool: String,

    /// Scripts for different build stages
    pub scripts: BuildScripts,
}

/// Build scripts for different environments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildScripts {
    /// Development build script
    pub dev: String,

    /// Production build script
    pub build: String,

    /// Test execution script
    pub test: String,
}

impl ProjectGenerationConfig {
    /// Create a new project generation configuration
    pub fn new(
        project_name: String,
        description: String,
        language: String,
        framework: String,
        project_type: ProjectType,
    ) -> Result<Self, String> {
        let config = Self {
            project_name,
            description,
            language,
            framework,
            project_type,
            technologies: Vec::new(),
            components: HashMap::new(),
            directory_structure: HashMap::new(),
            dependencies: DependencyConfig {
                production: HashMap::new(),
                development: HashMap::new(),
            },
            build_config: BuildConfig {
                build_tool: String::new(),
                scripts: BuildScripts {
                    dev: String::new(),
                    build: String::new(),
                    test: String::new(),
                },
            },
            initialization_commands: Vec::new(),
            recommendations: Vec::new(),
        };

        // Validate the configuration
        config.validate()?;

        Ok(config)
    }

    /// Validate the project generation configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate project name
        if !is_valid_project_name(&self.project_name) {
            return Err("Project name must be in kebab-case format".to_string());
        }

        // Validate language
        if self.language.trim().is_empty() {
            return Err("Language must be specified".to_string());
        }

        // Validate project type
        match self.project_type {
            ProjectType::WebApplication => {
                if self.framework.trim().is_empty() {
                    return Err("Web applications must specify a framework".to_string());
                }
            }
            _ => {}
        }

        // Validate components
        for (component_name, responsibility) in &self.components {
            if component_name.trim().is_empty() || responsibility.trim().is_empty() {
                return Err("Component name and responsibility must not be empty".to_string());
            }
        }

        // Validate directory structure
        if self.directory_structure.is_empty() {
            return Err("Directory structure must be defined".to_string());
        }

        Ok(())
    }

    /// Add a production dependency
    pub fn add_production_dependency(&mut self, name: &str, version: &str) {
        self.dependencies.production.insert(name.to_string(), version.to_string());
    }

    /// Add a development dependency
    pub fn add_development_dependency(&mut self, name: &str, version: &str) {
        self.dependencies.development.insert(name.to_string(), version.to_string());
    }

    /// Set build scripts
    pub fn set_build_scripts(&mut self, dev: &str, build: &str, test: &str) -> Result<(), String> {
        if dev.trim().is_empty() || build.trim().is_empty() || test.trim().is_empty() {
            return Err("All build scripts must be specified".to_string());
        }

        self.build_config.scripts = BuildScripts {
            dev: dev.to_string(),
            build: build.to_string(),
            test: test.to_string(),
        };

        // Set appropriate build tool based on language
        self.build_config.build_tool = match self.language.to_lowercase().as_str() {
            "rust" => "cargo".to_string(),
            "javascript" | "typescript" => "npm".to_string(),
            "python" => "pip".to_string(),
            _ => return Err(format!("Unsupported language: {}", self.language)),
        };

        Ok(())
    }

    /// Add initialization command
    pub fn add_initialization_command(&mut self, command: &str) {
        self.initialization_commands.push(command.to_string());
    }

    /// Add a recommendation
    pub fn add_recommendation(&mut self, recommendation: &str) {
        self.recommendations.push(recommendation.to_string());
    }

    /// Add a component with its responsibility
    pub fn add_component(&mut self, name: &str, responsibility: &str) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Component name cannot be empty".to_string());
        }
        if responsibility.trim().is_empty() {
            return Err("Component responsibility cannot be empty".to_string());
        }

        self.components.insert(name.to_string(), responsibility.to_string());
        
        // Automatically create directory structure for the component
        let mut files = Vec::new();
        match self.language.to_lowercase().as_str() {
            "rust" => {
                files.push("mod.rs".to_string());
                files.push("tests.rs".to_string());
            }
            "javascript" | "typescript" => {
                files.push("index.js".to_string());
                files.push("__tests__/index.test.js".to_string());
            }
            "python" => {
                files.push("__init__.py".to_string());
                files.push("test_component.py".to_string());
            }
            _ => {}
        }
        
        self.directory_structure.insert(name.to_string(), files);
        Ok(())
    }

    /// Add a technology
    pub fn add_technology(&mut self, technology: &str) -> Result<(), String> {
        if technology.trim().is_empty() {
            return Err("Technology cannot be empty".to_string());
        }
        self.technologies.push(technology.to_string());
        Ok(())
    }

    /// Generate a sample project configuration for testing
    pub fn sample_web_project() -> Self {
        let mut config = Self::new(
            "task-tracker".to_string(),
            "A simple task tracking application".to_string(),
            "rust".to_string(),
            "actix-web".to_string(),
            ProjectType::WebApplication,
        ).unwrap();

        // Add components
        config.add_component("api", "RESTful API endpoints for task management").unwrap();
        config.add_component("auth", "Authentication and authorization service").unwrap();
        config.add_component("database", "Database access and models").unwrap();
        config.add_component("utils", "Common utilities and helper functions").unwrap();

        // Production dependencies
        config.add_production_dependency("actix-web", "4.3.1");
        config.add_production_dependency("serde", "1.0.193");
        config.add_production_dependency("serde_json", "1.0.108");
        config.add_production_dependency("sqlx", "0.7.3");
        config.add_production_dependency("tokio", "1.35.1");

        // Development dependencies
        config.add_development_dependency("mockall", "0.11.4");
        config.add_development_dependency("cargo-watch", "8.5.2");
        config.add_development_dependency("clippy", "0.1.77");

        // Directory structure
        config.directory_structure = HashMap::from([
            ("src".to_string(), vec![
                "main.rs".to_string(),
                "api/mod.rs".to_string(),
                "auth/mod.rs".to_string(),
                "database/mod.rs".to_string(),
                "utils/mod.rs".to_string(),
            ]),
            ("tests".to_string(), vec![
                "api_tests.rs".to_string(),
                "auth_tests.rs".to_string(),
                "database_tests.rs".to_string(),
            ]),
            ("migrations".to_string(), vec![
                "initial_schema.sql".to_string()
            ]),
        ]);

        // Set build scripts
        config.set_build_scripts(
            "cargo watch -x run",
            "cargo build --release",
            "cargo test",
        ).unwrap();

        // Add initialization commands
        config.add_initialization_command("cargo new task-tracker");
        config.add_initialization_command("cd task-tracker");
        config.add_initialization_command("cargo add actix-web serde serde_json sqlx tokio");
        config.add_initialization_command("cargo add --dev mockall cargo-watch clippy");

        // Add recommendations
        config.add_recommendation("Use SQLx migrations for database schema management");
        config.add_recommendation("Implement proper error handling for each component");
        config.add_recommendation("Add comprehensive tests for each component");
        config.add_recommendation("Use environment variables for configuration");

        config
    }
}

fn is_valid_project_name(name: &str) -> bool {
    // Check if the name is in kebab-case format
    let is_kebab_case = name.chars().all(|c| c.is_ascii_lowercase() || c == '-');
    let no_consecutive_hyphens = !name.contains("--");
    let valid_start_end = !name.starts_with('-') && !name.ends_with('-');
    
    is_kebab_case && no_consecutive_hyphens && valid_start_end
}

// Type aliases for backward compatibility
pub use crate::prompt::{
    ProjectConfig,
    ProjectType,
    DependencyConfig,
    BuildConfig,
    BuildScripts,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_generation_config_creation() {
        let mut config = ProjectGenerationConfig::new(
            "task-manager".to_string(),
            "A simple task management application".to_string(),
            "rust".to_string(),
            "actix-web".to_string(),
            ProjectType::WebApplication,
        ).unwrap();

        config.add_production_dependency("actix-web", "4.0.1");
        config.add_development_dependency("mockall", "0.11.0");
        config.set_build_scripts(
            "cargo run", 
            "cargo build --release", 
            "cargo test"
        ).unwrap();
        config.add_initialization_command("cargo new task-manager");
        config.add_recommendation("Use environment variables for configuration");

        assert_eq!(config.project_name, "task-manager");
        assert_eq!(config.dependencies.production.get("actix-web"), Some(&"4.0.1".to_string()));
        assert_eq!(config.build_config.scripts.test, "cargo test");
    }
}
