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

    #[test]
    fn test_project_config_creation() -> Result<()> {
        let project_config = ProjectConfig {
            name: "test_project".to_string(),
            description: Some("A test project".to_string()),
            technologies: vec!["rust".to_string()],
            project_type: ProjectType::Application,
            language: "rust".to_string(),
            framework: Some("actix-web".to_string()),
            dependencies: Some(DependencyConfig {
                required: vec!["serde".to_string()],
                optional: vec!["tokio".to_string()],
                development: vec!["mockall".to_string()],
            }),
            build_config: None,
            directory_structure: None,
            initialization_commands: None,
            recommendations: None,
        };

        assert_eq!(project_config.name, "test_project");
        assert_eq!(project_config.description, Some("A test project".to_string()));
        assert_eq!(project_config.technologies, vec!["rust".to_string()]);
        assert!(matches!(project_config.project_type, ProjectType::Application));

        Ok(())
    }

    #[test]
    fn test_project_config_minimal() -> Result<()> {
        let project_config = ProjectConfig {
            name: "test_task".to_string(),
            description: None,
            technologies: vec![],
            project_type: ProjectType::Library,
            language: "python".to_string(),
            framework: None,
            dependencies: None,
            build_config: None,
            directory_structure: None,
            initialization_commands: None,
            recommendations: None,
        };

        assert_eq!(project_config.name, "test_task");
        assert_eq!(project_config.description, None);
        assert!(project_config.technologies.is_empty());
        assert!(matches!(project_config.project_type, ProjectType::Library));

        Ok(())
    }

    #[tokio::test]
    async fn test_project_config_technologies() -> Result<()> {
        let project_config = ProjectConfig {
            name: "test_task".to_string(),
            description: None,
            technologies: vec!["Rust".to_string(), "Tokio".to_string()],
            project_type: ProjectType::Library,
            language: "rust".to_string(),
            framework: None,
            dependencies: None,
            build_config: None,
            directory_structure: None,
            initialization_commands: None,
            recommendations: None,
        };

        assert_eq!(project_config.name, "test_task");
        assert!(project_config.technologies.contains(&"Rust".to_string()));
        Ok(())
    }
}
