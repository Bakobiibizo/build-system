use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs as tokio_fs;
use tokio::io::AsyncWriteExt;
use crate::inference::InferenceClient;
use crate::state::types::TaskId;

mod project_generation;
pub use project_generation::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    pub name: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub resources: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildStep {
    pub task_config: TaskConfig,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Prompt {
    pub system_context: String,
    pub user_request: String,
    pub build_context: Option<String>,
}

#[async_trait]
pub trait PromptProcessor: Send + Sync {
    async fn process_response(&self, response: String) -> Result<()>;
}

#[derive(Debug)]
pub struct PromptManager {
    template_dir: String,
    templates: HashMap<String, String>,
}

impl PromptManager {
    pub fn new(template_dir: String) -> Result<Self> {
        let mut templates = HashMap::new();
        
        // Try to load project generation template
        let template_path = PathBuf::from(&template_dir).join("project_generation_prompt.md");
        if template_path.exists() {
            let template_content = std::fs::read_to_string(&template_path)
                .context("Failed to read project generation template")?;
            templates.insert("project_generation_prompt".to_string(), template_content);
        }

        Ok(Self {
            templates,
            template_dir,
        })
    }

    pub async fn create_prompt(
        &self,
        user_request: String,
        build_context: Option<String>,
    ) -> Result<Prompt> {
        let system_context = self.get_system_context()?;
        Ok(Prompt {
            system_context,
            user_request,
            build_context,
        })
    }

    fn get_system_context(&self) -> Result<String> {
        self.templates
            .get("system")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("System prompt template not found"))
    }

    pub fn interpret_task(&self, task_str: &str) -> Result<TaskConfig> {
        let mut name = String::new();
        let mut description = String::new();
        let mut dependencies = Vec::new();
        let mut resources = HashMap::new();

        for line in task_str.lines() {
            let line = line.trim();
            if line.starts_with("Build task") {
                name = line["Build task".len()..].trim().to_string();
            } else if line.starts_with("Depends:") {
                dependencies = line["Depends:".len()..]
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
            } else if line.starts_with("Resource:") {
                let parts: Vec<&str> = line["Resource:".len()..].split('=').collect();
                if parts.len() == 2 {
                    resources.insert(
                        parts[0].trim().to_string(),
                        parts[1].trim().to_string(),
                    );
                }
            } else {
                description.push_str(line);
                description.push('\n');
            }
        }

        Ok(TaskConfig {
            name,
            description: description.trim().to_string(),
            dependencies,
            resources,
        })
    }

    pub fn generate_build_steps(&self, config: &TaskConfig) -> Result<Vec<BuildStep>> {
        let step = BuildStep {
            task_config: TaskConfig {
                name: config.name.clone(),
                description: config.description.clone(),
                dependencies: config.dependencies.clone(),
                resources: config.resources.clone(),
            },
            command: format!("echo 'Building {}'", config.name),
            args: vec![],
            env: HashMap::new(),
        };
        Ok(vec![step])
    }

    /// Process a project generation request
    pub async fn generate_project(&self, user_request: &str) -> Result<ProjectConfig> {
        // Retrieve the project generation template
        let template = self.templates.get("project_generation_prompt")
            .cloned()
            .unwrap_or_else(|| "Generate a project configuration based on the user's request.".to_string());

        // Initialize inference client
        let inference_client = InferenceClient::new()?;

        // Prepare AI prompt for project generation
        let system_context = format!(
            "{}\n\nUser Request: {}",
            template,
            user_request
        );

        let inference_prompt = Prompt {
            system_context,
            user_request: "Generate a comprehensive project configuration in JSON format.".to_string(),
            build_context: None,
        };

        // Use AI to generate project configuration
        let (response, _status) = inference_client
            .execute_task_prompt(&inference_prompt, &TaskId::new("project_generation"))
            .await?;

        // Parse AI response into ProjectConfig
        let config = self.parse_project_config(response).await?;

        // Create project structure based on AI-generated config
        self.create_project_structure(&config).await?;

        Ok(config)
    }

    /// Create project directory and files based on configuration
    async fn create_project_structure(&self, config: &ProjectConfig) -> Result<()> {
        let project_root = PathBuf::from(&config.project_name);
        
        // Create root project directory
        tokio_fs::create_dir_all(&project_root).await?;

        // Create directory structure
        for (dir_name, files) in &config.directory_structure.root {
            let dir_path = project_root.join(dir_name);
            tokio_fs::create_dir_all(&dir_path).await?;

            // Create placeholder files
            for file_name in files {
                let file_path = dir_path.join(file_name);
                let mut file = tokio_fs::File::create(file_path).await?;
                
                // Add some basic content based on file type
                let content = match file_name.as_str() {
                    "main.rs" => format!(
                        "use actix_web::{{web, App, HttpResponse, HttpServer, Responder}};\n\n\
                        #[actix_web::main]\n\
                        async fn main() -> std::io::Result<()> {{\n\
                            HttpServer::new(|| {{\n\
                                App::new()\n\
                                    .route(\"/\", web::get().to(index))\n\
                            }})\n\
                            .bind(\"127.0.0.1:8080\")?\n\
                            .run()\n\
                            .await\n\
                        }}\n\n\
                        async fn index() -> impl Responder {{\n\
                            HttpResponse::Ok().body(\"{} Web Application\".to_string())\n\
                        }}\n", 
                        config.project_name
                    ),
                    "routes.rs" => "// API routes will be defined here\n".to_string(),
                    "models.rs" => "// Data models will be defined here\n".to_string(),
                    "db.rs" => "// Database connection and queries will be defined here\n".to_string(),
                    "integration_tests.rs" => "// Integration tests will be defined here\n".to_string(),
                    "20240128_create_tasks_table.sql" => 
                        "CREATE TABLE IF NOT EXISTS tasks (\n\
                            id SERIAL PRIMARY KEY,\n\
                            title VARCHAR(255) NOT NULL,\n\
                            description TEXT,\n\
                            status VARCHAR(50) DEFAULT 'pending'\n\
                        );\n".to_string(),
                    _ => String::new(),
                };

                file.write_all(content.as_bytes()).await?;
            }
        }

        // Create Cargo.toml
        let cargo_toml_path = project_root.join("Cargo.toml");
        let mut cargo_toml = tokio_fs::File::create(cargo_toml_path).await?;
        let cargo_toml_content = format!(
            "[package]\n\
            name = \"{}\"\n\
            version = \"0.1.0\"\n\
            edition = \"2021\"\n\n\
            [dependencies]\n{}\n\n\
            [dev-dependencies]\n{}\n",
            config.project_name,
            config.dependencies.production.iter()
                .map(|(k, v)| format!("{} = \"{}\"\n", k, v))
                .collect::<String>(),
            config.dependencies.development.iter()
                .map(|(k, v)| format!("{} = \"{}\"\n", k, v))
                .collect::<String>()
        );
        cargo_toml.write_all(cargo_toml_content.as_bytes()).await?;

        // Create README.md
        let readme_path = project_root.join("README.md");
        let mut readme = tokio_fs::File::create(readme_path).await?;
        let readme_content = format!(
            "# {}\n\n\
            ## Project Overview\n\
            A {} {} built with {}.\n\n\
            ## Getting Started\n\
            ```bash\n\
            {} # Initialization command\n\
            {} # Install dependencies\n\
            {} # Run development server\n\
            ```\n\n\
            ## Recommendations\n{}\n",
            config.project_name,
            config.project_type,
            config.project_name,
            config.framework,
            config.initialization_commands.get(0).cloned().unwrap_or_default(),
            config.initialization_commands.get(1).cloned().unwrap_or_default(),
            config.initialization_commands.get(2).cloned().unwrap_or_default(),
            config.recommendations.join("\n- ")
        );
        readme.write_all(readme_content.as_bytes()).await?;

        Ok(())
    }

    async fn parse_project_config(&self, response: String) -> Result<ProjectConfig> {
        // Use anyhow's context for error handling
        let config: ProjectConfig = serde_json::from_str(&response)
            .with_context(|| format!("Failed to parse project configuration from AI response: {}", response))?;

        // Validate project configuration
        self.validate_project_config(&config)?;

        Ok(config)
    }

    fn validate_project_config(&self, config: &ProjectConfig) -> Result<()> {
        // Validate project name (kebab-case)
        if config.project_name.is_empty() {
            return Err(anyhow::anyhow!("Project name cannot be empty"));
        }

        // Validate language and framework naming conventions
        if !config.language.chars().all(|c| c.is_lowercase() || c == '-') {
            return Err(anyhow::anyhow!("Language must be lowercase"));
        }

        if !config.framework.chars().all(|c| c.is_lowercase() || c == '-') {
            return Err(anyhow::anyhow!("Framework must be lowercase"));
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectConfig {
    pub project_name: String,
    pub language: String,
    pub framework: String,
    pub project_type: String,
    pub directory_structure: DirectoryStructure,
    pub dependencies: ProjectDependencies,
    pub build_config: BuildConfig,
    pub initialization_commands: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirectoryStructure {
    pub root: HashMap<String, Vec<String>>,
    pub src: Vec<String>,
    pub tests: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectDependencies {
    pub production: HashMap<String, String>,
    pub development: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildConfig {
    pub build_tool: String,
    pub scripts: BuildScripts,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildScripts {
    pub dev: String,
    pub build: String,
    pub test: String,
}

#[async_trait]
impl PromptProcessor for PromptManager {
    async fn process_response(&self, response: String) -> Result<()> {
        let config = self.interpret_task(&response)?;
        let build_steps = self.generate_build_steps(&config)?;
        
        // In a real system, you might want to execute these build steps
        for step in build_steps {
            println!("Executing build step: {:?}", step);
        }
        
        Ok(())
    }
}
