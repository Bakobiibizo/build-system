use build_system::inference::InferenceClient;
use build_system::prompt::Prompt;
use build_system::state::types::TaskId;

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
        
        let prompt = Prompt {
            system_context: "You are a helpful assistant".to_string(),
            user_request: "Generate a test response".to_string(),
            build_context: None,
        };

        let task_id = TaskId::new("test-task-1");

        let result = client.execute_task_prompt(&prompt, &task_id).await;
        assert!(result.is_ok());
    }
}
