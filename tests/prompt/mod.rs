use anyhow::Result;
use build_system::prompt::{PromptManager, TaskConfig};
use mockall::automock;
use std::collections::HashMap;
use tempfile::TempDir;
use tokio::fs;
use async_trait::async_trait;

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::eq;

    #[automock]
    #[async_trait]
    pub trait TestPromptProcessor: Send + Sync {
        async fn process_response(&self, response: String) -> Result<()>;
    }

    #[async_trait]
    impl TestPromptProcessor for PromptManager {
        async fn process_response(&self, _response: String) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_prompt_manager_creation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let template_dir = temp_dir.path().to_string_lossy().to_string();
        
        // Create a test template
        fs::write(
            temp_dir.path().join("system.prompt"),
            "You are a test build system assistant.",
        ).await?;
        
        let manager = PromptManager::new(template_dir).await;
        let prompt = manager.create_prompt("test request".to_string(), None).await?;
        
        assert_eq!(prompt.system_context, "You are a test build system assistant.");
        assert_eq!(prompt.user_request, "test request");
        assert!(prompt.build_context.is_none());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_task_interpretation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let template_dir = temp_dir.path().to_string_lossy().to_string();
        let manager = PromptManager::new(template_dir).await;
        
        let request = r#"
        Build task test_task
        Depends: task1, task2
        Resource: memory=2GB
        Resource: cpu=4
        This is a test task description
        "#;
        
        let config = manager.interpret_task(request)?;
        
        assert_eq!(config.name, "test_task");
        assert_eq!(config.dependencies, vec!["task1", "task2"]);
        assert_eq!(config.resources.get("memory"), Some(&"2GB".to_string()));
        assert_eq!(config.resources.get("cpu"), Some(&"4".to_string()));
        assert!(config.description.contains("This is a test task description"));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_build_step_generation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let template_dir = temp_dir.path().to_string_lossy().to_string();
        let manager = PromptManager::new(template_dir).await;
        
        let mut resources = HashMap::new();
        resources.insert("memory".to_string(), "2GB".to_string());
        
        let config = TaskConfig {
            name: "test_task".to_string(),
            description: "Test task".to_string(),
            dependencies: vec!["dep1".to_string()],
            resources,
        };
        
        let steps = manager.generate_build_steps(&config)?;
        
        assert!(!steps.is_empty());
        assert_eq!(steps[0].task_config.name, "test_task");
        assert_eq!(steps[0].command, "echo 'Building test_task'");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_prompt_processing() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let template_dir = temp_dir.path().to_string_lossy().to_string();
        let manager = PromptManager::new(template_dir).await;
        
        let response = r#"
        Build task test_task
        Depends: task1
        Resource: memory=1GB
        Test task description
        "#;
        
        // Test that processing doesn't fail
        TestPromptProcessor::process_response(&manager, response.to_string()).await?;
        
        Ok(())
    }

    #[tokio::test]
    async fn test_prompt_processor_mock() -> Result<()> {
        let mut mock = MockTestPromptProcessor::new();
        
        mock.expect_process_response()
            .with(eq("test response".to_string()))
            .times(1)
            .returning(|_| Ok(()));
        
        mock.process_response("test response".to_string()).await?;
        
        Ok(())
    }
}
