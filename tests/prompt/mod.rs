use anyhow::Result;
use std::collections::HashMap;
use build_system::prompt::{
    Prompt, 
    PromptManager, 
    ProjectConfig,
    ProjectType,
    DependencyConfig
};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_prompt_creation() -> Result<()> {
        let prompt = Prompt::new(
            "You are an AI assistant",
            "Help me create a project"
        );

        assert_eq!(prompt.system_context, "You are an AI assistant");
        assert_eq!(prompt.user_request, "Help me create a project");
        Ok(())
    }

    #[tokio::test]
    async fn test_prompt_manager_initialization() -> Result<()> {
        let _prompt_manager = PromptManager::new("./templates")?;
        Ok(())
    }

    #[tokio::test]
    async fn test_project_config_creation() -> Result<()> {
        let mut dependencies = HashMap::new();
        dependencies.insert("production_dep1".to_string(), "version1".to_string());
        dependencies.insert("development_dep1".to_string(), "version2".to_string());

        let dependency_config = DependencyConfig {
            production: dependencies.clone(),
            development: HashMap::new(),
        };

        let project_config = ProjectConfig {
            project_name: "test_project".to_string(),
            project_type: Some(ProjectType::WebApplication),
            description: Some("A test project".to_string()),
            dependencies: Some(dependency_config),
            ..Default::default()
        };

        assert_eq!(project_config.project_name, "test_project");
        assert!(project_config.project_type.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_project_config_technologies() -> Result<()> {
        let project_config = ProjectConfig {
            project_name: "test_task".to_string(),
            technologies: vec!["Rust".to_string(), "Tokio".to_string()],
            ..Default::default()
        };
        
        assert_eq!(project_config.project_name, "test_task");
        assert!(project_config.technologies.is_sorted());
        Ok(())
    }
}
