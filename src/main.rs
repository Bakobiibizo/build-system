use anyhow::Result;
use dotenv::dotenv;
use tracing::{info, error};

mod state;
mod prompt;
mod inference;

use crate::inference::{InferenceClient, InferenceProcessor};
use crate::prompt::{Prompt, PromptManager};
use crate::state::types::TaskId;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize InferenceClient
    let inference_client = InferenceClient::new()?;

    // Initialize PromptManager
    let template_dir = "./src/prompt".to_string();
    let prompt_manager = PromptManager::new(template_dir)?;

    // Demonstration prompts
    let demo_prompts = vec![
        "Generate a comprehensive project structure for a task management web application",
        "Create a design for a real-time chat application with WebSocket support",
        "Outline an architecture for a machine learning model deployment platform"
    ];

    // Demonstrate different inference methods
    for (index, prompt_text) in demo_prompts.iter().enumerate() {
        info!("Demonstration {} - Prompt: {}", index + 1, prompt_text);

        // Prepare prompt
        let prompt = Prompt {
            system_context: "You are an expert software architect.".to_string(),
            user_request: prompt_text.to_string(),
            build_context: None,
        };

        // Execute task prompt
        match inference_client.execute_task_prompt(&prompt, &TaskId::new("demo_task")).await {
            Ok((response, status)) => {
                info!("Task Status: {:?}", status);
                info!("AI Response:\n{}", response);

                // Optional: Parse and use the AI-generated project configuration
                match prompt_manager.generate_project(prompt_text).await {
                    Ok(project_config) => {
                        info!("Generated Project Configuration: {:?}", project_config);
                    }
                    Err(e) => {
                        error!("Failed to generate project configuration: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Inference failed: {}", e);
            }
        }

        // Optional: Demonstrate streaming completion
        info!("Streaming Completion Demonstration:");
        match inference_client.stream_completion(prompt_text, 0.6).await {
            Ok(streamed_response) => {
                info!("Streamed Response Length: {} characters", streamed_response.len());
            }
            Err(e) => {
                error!("Streaming failed: {}", e);
            }
        }

        // Add a small delay between demonstrations
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    Ok(())
}
