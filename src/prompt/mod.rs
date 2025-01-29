use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs as tokio_fs;
use tokio::io::AsyncWriteExt;
use serde::{Serialize, Deserialize};

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
    pub async fn new(template_dir: String) -> Self {
        let mut templates = HashMap::new();
        
        // Create directory if it doesn't exist
        if tokio_fs::metadata(&template_dir).await.is_err() {
            tokio_fs::create_dir_all(&template_dir).await.unwrap();
        }
        
        let mut dir = tokio_fs::read_dir(&template_dir).await.unwrap();

        while let Ok(Some(entry)) = dir.next_entry().await {
            if let Ok(file_type) = entry.file_type().await {
                if file_type.is_file() {
                    if let Some(extension) = entry.path().extension() {
                        if extension == "prompt" {
                            if let Ok(content) = tokio_fs::read_to_string(entry.path()).await {
                                if let Some(name) = entry.path().file_stem() {
                                    templates.insert(
                                        name.to_string_lossy().to_string(),
                                        content,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        Self {
            template_dir,
            templates,
        }
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
    pub async fn generate_project(&self, _user_request: &str) -> Result<ProjectGenerationConfig> {
        // For now, we'll use a sample project configuration
        // In a real implementation, this would use an AI model to generate based on user_request
        let config = ProjectGenerationConfig::sample_web_project();
        
        // Create project directory and files
        self.create_project_structure(&config).await?;
        
        Ok(config)
    }

    /// Create project directory and files based on configuration
    async fn create_project_structure(&self, config: &ProjectGenerationConfig) -> Result<()> {
        let project_root = PathBuf::from(&config.project_name);
        
        // Create root project directory
        tokio_fs::create_dir_all(&project_root).await?;

        // Create directory structure
        for (dir_name, files) in &config.directory_structure {
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
            format!("{:?}", config.project_type),
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
