use std::env;
use mockall::{mock, predicate};
use tokio;

use crate::state::types::TaskId;
use crate::inference::InferenceClient;
use crate::prompt::Prompt;

// Define HttpClientTrait explicitly
#[async_trait::async_trait]
pub trait HttpClientTrait {
    async fn post(&self, url: &str) -> anyhow::Result<String>;
}

// Use the new mock! syntax
mock! {
    HttpClient {}
    
    #[async_trait::async_trait]
    impl HttpClientTrait for HttpClient {
        async fn post(&self, url: &str) -> anyhow::Result<String>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to set up test environment variables
    fn setup_test_env() {
        env::set_var("INFERENCE_API_URL", "https://test-api.com");
        env::set_var("INFERENCE_API_KEY", "test-key");
        env::set_var("INFERENCE_API_MODEL", "test-model");
        env::set_var("INFERENCE_API_TEMPERATURE", "0.1");
    }

    #[tokio::test]
    async fn test_inference_client_initialization() {
        // Setup test environment variables
        env::set_var("INFERENCE_API_KEY", "test-key");
        env::set_var("INFERENCE_API_BASE_URL", "https://test-api.com");
        
        let client = InferenceClient::new();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_mock_http_client() {
        let mut mock_client = MockHttpClient::new();
        
        mock_client
            .expect_post()
            .with(predicate::eq("https://test-url.com"))
            .returning(|_| Ok("Test response".to_string()));
        
        let result = mock_client.post("https://test-url.com").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Test response");
    }

    #[tokio::test]
    async fn test_execute_task_prompt_success() {
        // Setup test environment variables
        env::set_var("INFERENCE_API_KEY", "test-key");
        env::set_var("INFERENCE_API_BASE_URL", "https://test-api.com");

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

    #[tokio::test]
    async fn test_execute_task_prompt_empty_response() {
        // Setup test environment variables
        env::set_var("INFERENCE_API_KEY", "test-key");
        env::set_var("INFERENCE_API_BASE_URL", "https://test-api.com");

        let client = InferenceClient::new().expect("Failed to create inference client");
        
        let prompt = Prompt {
            system_context: "You are a helpful assistant".to_string(),
            user_request: "Generate an empty response".to_string(),
            build_context: None,
        };

        let task_id = TaskId::new("test-task-2");

        let result = client.execute_task_prompt(&prompt, &task_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[should_panic(expected = "environment variable not found")]
    async fn test_inference_client_missing_env_var() {
        // Clear all test environment variables
        env::remove_var("INFERENCE_API_URL");
        env::remove_var("INFERENCE_API_KEY");
        env::remove_var("INFERENCE_API_MODEL");
        env::remove_var("INFERENCE_API_TEMPERATURE");

        // This should panic due to missing environment variables
        let _ = InferenceClient::new().expect("Should panic on missing env vars");
    }
}
