use anyhow::Result;
use crate::prompt::Prompt;
use mockall::mock;
use async_trait::async_trait;

// Define HttpClientTrait explicitly
#[async_trait]
pub trait HttpClientTrait {
    async fn post(&self, url: &str) -> anyhow::Result<String>;
}

mock! {
    pub HttpClient {}

    #[async_trait]
    impl HttpClientTrait for HttpClient {
        async fn post(&self, url: &str) -> anyhow::Result<String>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
