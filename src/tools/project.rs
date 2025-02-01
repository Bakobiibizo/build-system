use anyhow::Result;
use serde_json::json;
use crate::{
    tools::{Tool, ExecutableTool},
    prompt::{PromptManager, ProjectConfig},
};
use async_trait::async_trait;
use tokio::runtime::Runtime;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::collections::HashMap;

static RUNTIME: Lazy<Mutex<Runtime>> = Lazy::new(|| {
    Mutex::new(Runtime::new().expect("Failed to create Tokio runtime"))
});

#[derive(Debug, Default)]
pub struct ProjectTool;

#[async_trait]
impl ExecutableTool for ProjectTool {
    async fn execute(&self, arguments: &str) -> Result<String, String> {
        // Parse the arguments
        let args: serde_json::Value = serde_json::from_str(arguments)
            .map_err(|e| format!("Failed to parse arguments: {}", e))?;
        
        // Check if we have a project design or need to generate one
        let config = if let Some(design_str) = args["design"].as_str() {
            // Use provided design
            serde_json::from_str(design_str)
                .map_err(|e| format!("Failed to parse project design: {}", e))?
        } else {
            // Generate design using prompt manager
            let name = args["name"].as_str()
                .ok_or("Missing project name")?;
            let language = args["language"].as_str()
                .ok_or("Missing project language")?;
            let description = args["description"].as_str()
                .unwrap_or("");

            // Create the user request
            let request = format!(
                "Create a {} project named '{}' with the following description: {}",
                language, name, description
            );

            // Initialize prompt manager with templates
            let prompt_manager = PromptManager::new("src/prompt/templates")
                .map_err(|e| format!("Failed to create prompt manager: {}", e))?;
            
            // Generate project configuration
            prompt_manager.generate_project_config(&request)
                .await
                .map_err(|e| format!("Failed to generate project config: {}", e))?
        };
        
        // Generate the project structure
        let project_root = format!("build/{}", config.name);
        tokio::fs::create_dir_all(&project_root)
            .await
            .map_err(|e| format!("Failed to create project directory: {}", e))?;

        // Create directory structure
        if let Some(structure) = config.directory_structure {
            for (dir, files) in structure {
                let dir_path = format!("{}/{}", project_root, dir);
                tokio::fs::create_dir_all(&dir_path)
                    .await
                    .map_err(|e| format!("Failed to create directory {}: {}", dir, e))?;
                
                for file in files {
                    let file_path = format!("{}/{}", dir_path, file);
                    tokio::fs::write(&file_path, "")
                        .await
                        .map_err(|e| format!("Failed to create file {}: {}", file, e))?;
                }
            }
        }

        // Create dependency files
        if let Some(deps) = config.dependencies {
            self.generate_requirements(&deps, &project_root)
                .map_err(|e| format!("Failed to generate requirements: {}", e))?;
        }

        // Create build configuration
        if let Some(build_config) = config.build_config {
            let build_json = serde_json::to_string_pretty(&build_config)
                .map_err(|e| format!("Failed to serialize build config: {}", e))?;
            
            tokio::fs::write(format!("{}/build.json", project_root), build_json)
                .await
                .map_err(|e| format!("Failed to create build.json: {}", e))?;
        }

        // Initialize git repository
        tokio::process::Command::new("git")
            .args(&["init"])
            .current_dir(&project_root)
            .output()
            .await
            .map_err(|e| format!("Failed to initialize git repository: {}", e))?;

        Ok(format!("Project generated successfully in {}", project_root))
    }

    fn get_tool_definition(&self) -> Tool {
        Tool {
            name: "project".to_string(),
            description: "Generate a new project using AI-powered design".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name of the project (in kebab-case)"
                    },
                    "language": {
                        "type": "string",
                        "description": "Programming language for the project"
                    },
                    "description": {
                        "type": "string",
                        "description": "Optional description of the project requirements and features"
                    },
                    "design": {
                        "type": "string",
                        "description": "Optional JSON string containing a complete project design. If not provided, the design will be generated using AI based on the project generation template."
                    }
                },
                "required": ["name", "language"]
            }),
        }
    }

    fn get_short_description(&self) -> String {
        "Generate a new project using AI-powered design based on project templates".to_string()
    }

    fn get_long_description(&self) -> String {
        r#"This tool generates a new project using AI-powered design based on project templates.

You can either:
1. Provide just the project name, language, and optional description - the tool will use AI to generate a complete project design following the project generation template
2. Provide a complete project design JSON if you want full control over the project structure

When using AI-powered generation, the tool will:
1. Use the project generation template to create a comprehensive project configuration
2. Consider best practices for software development
3. Generate a realistic and implementable project structure
4. Include appropriate dependencies and build configuration
5. Set up the project following the specified template

The generated project will include:
- Proper directory structure
- Production and development dependencies
- Build configuration
- Deployment settings
- Initialization commands
- Best practice recommendations

Example usage with AI generation:
{
    "name": "my-flask-app",
    "language": "python",
    "description": "A web application with user authentication, SQLite database, and RESTful API endpoints"
}

Example usage with manual design:
{
    "name": "my-project",
    "language": "python",
    "design": {
        "name": "my-project",
        "description": "Simple Flask web app",
        "technologies": ["python", "flask", "sqlite"],
        "project_type": "Application",
        "language": "python",
        "framework": "flask",
        "dependencies": {
            "production": {
                "flask": "2.0.1",
                "sqlalchemy": "1.4.23"
            },
            "development": {
                "pytest": "6.2.5"
            }
        },
        "build_config": {
            "build_tool": "pip",
            "scripts": {
                "start": "flask run",
                "test": "pytest",
                "build": "pip install -r requirements.txt"
            }
        },
        "directory_structure": {
            "src": ["app.py", "models.py", "routes.py"],
            "tests": ["test_app.py"],
            "migrations": ["initial.sql"]
        }
    }
}"#.to_string()
    }
}

// Separate implementation block for ProjectTool-specific methods
impl ProjectTool {
    fn generate_requirements(&self, deps: &HashMap<String, HashMap<String, String>>, project_root: &str) -> Result<()> {
        // Create requirements.txt
        if let Some(production) = deps.get("production") {
            let requirements = production.iter()
                .map(|(name, version)| format!("{}=={}", name, version))
                .collect::<Vec<_>>()
                .join("\n");
            
            std::fs::write(format!("{}/requirements.txt", project_root), requirements)?;
        }

        // Create dev-requirements.txt
        if let Some(development) = deps.get("development") {
            let dev_requirements = development.iter()
                .map(|(name, version)| format!("{}=={}", name, version))
                .collect::<Vec<_>>()
                .join("\n");
            
            std::fs::write(format!("{}/dev-requirements.txt", project_root), dev_requirements)?;
        }

        Ok(())
    }
}
