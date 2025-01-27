use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::future::Future;
use std::pin::Pin;
use tokio::fs as tokio_fs;
use serde::{Serialize, Deserialize};

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

#[async_trait]
impl PromptProcessor for PromptManager {
    async fn process_response(&self, response: String) -> Result<()> {
        // Process the response
        let config = self.interpret_task(&response)?;
        let _steps = self.generate_build_steps(&config)?;
        Ok(())
    }
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
}
