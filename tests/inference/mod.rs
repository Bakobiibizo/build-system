use build_system::inference::InferenceClient;
use build_system::prompt::Prompt;
use build_system::state::types::TaskId;
use anyhow::Result;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_inference_client_initialization() {
        // Setup test environment variables
        std::env::set_var("INFERENCE_API_KEY", "test-key");
        std::env::set_var("INFERENCE_API_BASE_URL", "https://test-api.com");
        
        let client = InferenceClient::new();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_execute_task_prompt() {
        // Setup test environment variables
        std::env::set_var("INFERENCE_API_KEY", "test-key");
        std::env::set_var("INFERENCE_API_BASE_URL", "https://test-api.com");

        let client = InferenceClient::new().expect("Failed to create inference client");
        
        let prompt = Prompt::new(
            "You are a helpful assistant",
            "Generate a test response",
        );

        let task_id = TaskId::new("test-task-1");

        let result = client.execute_task_prompt(&prompt, &task_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_prompt_generation() -> Result<()> {
        let prompt = Prompt::new(
            "You are a helpful assistant",
            "Generate a test response",
        );

        assert_eq!(prompt.system_context, "You are a helpful assistant");
        assert_eq!(prompt.user_request, "Generate a test response");

        Ok(())
    }

    #[tokio::test]
    async fn test_prompt_serialization() -> Result<()> {
        let prompt = Prompt::new(
            "System context",
            "User request",
        );

        let serialized = serde_json::to_string(&prompt)?;
        let deserialized: Prompt = serde_json::from_str(&serialized)?;

        assert_eq!(prompt.system_context, deserialized.system_context);
        assert_eq!(prompt.user_request, deserialized.user_request);

        Ok(())
    }
}
