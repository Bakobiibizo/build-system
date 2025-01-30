use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export ProjectType from prompt/mod.rs
pub use crate::prompt::ProjectType;

/// Represents a comprehensive project generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectGenerationConfig {
    /// The name of the project in kebab-case
    pub project_name: String,

    /// Primary programming language
    pub language: String,

    /// Web framework or library
    pub framework: String,

    /// Type of project
    pub project_type: ProjectType,

    /// Detailed directory structure
    pub directory_structure: HashMap<String, Vec<String>>,

    /// Production dependencies
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
        language: String,
        framework: String,
        project_type: ProjectType,
    ) -> Self {
        Self {
            project_name,
            language,
            framework,
            project_type,
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
        }
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
    pub fn set_build_scripts(&mut self, dev: &str, build: &str, test: &str) {
        self.build_config.scripts = BuildScripts {
            dev: dev.to_string(),
            build: build.to_string(),
            test: test.to_string(),
        };
    }

    /// Add initialization command
    pub fn add_initialization_command(&mut self, command: &str) {
        self.initialization_commands.push(command.to_string());
    }

    /// Add a recommendation
    pub fn add_recommendation(&mut self, recommendation: &str) {
        self.recommendations.push(recommendation.to_string());
    }

    /// Generate a sample project configuration for testing
    pub fn sample_web_project() -> Self {
        let mut config = Self::new(
            "task-tracker".to_string(),
            "rust".to_string(),
            "actix-web".to_string(),
            ProjectType::WebApplication,
        );

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
                "routes.rs".to_string(), 
                "models.rs".to_string(), 
                "db.rs".to_string()
            ]),
            ("tests".to_string(), vec![
                "integration_tests.rs".to_string()
            ]),
            ("migrations".to_string(), vec![
                "20240128_create_tasks_table.sql".to_string()
            ])
        ]);

        // Build configuration
        config.build_config.build_tool = "cargo".to_string();
        config.set_build_scripts(
            "cargo watch -x run", 
            "cargo build --release", 
            "cargo test"
        );

        // Initialization commands
        config.add_initialization_command("cargo new task-tracker");
        config.add_initialization_command("cd task-tracker");
        config.add_initialization_command("cargo add actix-web serde sqlx");

        // Recommendations
        config.add_recommendation("Use environment variables for configuration");
        config.add_recommendation("Implement proper error handling");
        config.add_recommendation("Set up database migrations");

        config
    }
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
            "rust".to_string(),
            "actix-web".to_string(),
            ProjectType::WebApplication,
        );

        config.add_production_dependency("actix-web", "4.0.1");
        config.add_development_dependency("mockall", "0.11.0");
        config.set_build_scripts(
            "cargo run", 
            "cargo build --release", 
            "cargo test"
        );
        config.add_initialization_command("cargo new task-manager");
        config.add_recommendation("Use environment variables for configuration");

        assert_eq!(config.project_name, "task-manager");
        assert_eq!(config.dependencies.production.get("actix-web"), Some(&"4.0.1".to_string()));
        assert_eq!(config.build_config.scripts.test, "cargo test");
    }
}
