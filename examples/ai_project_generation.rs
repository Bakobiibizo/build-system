use anyhow::Result;
use build_system::inference::InferenceClient;
use build_system::prompt::Prompt;
use build_system::state::types::TaskId;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize AI inference client
    let inference_client = InferenceClient::new()?;

    // Define project generation prompts
    let project_prompts = vec![
        "Create a comprehensive project structure for a task management web application with user authentication",
        "Design a real-time collaborative document editing platform",
        "Develop a machine learning model deployment framework with CI/CD integration"
    ];

    // Iterate through project prompts and generate configurations
    for (index, prompt_text) in project_prompts.iter().enumerate() {
        println!("\n--- Project Generation {} ---", index + 1);
        println!("Prompt: {}", prompt_text);

        // Prepare the prompt
        let prompt = Prompt {
            system_context: "You are an expert software architect specializing in modern, scalable application design.".to_string(),
            user_request: prompt_text.to_string(),
            build_context: None,
        };

        // Execute AI-powered project generation
        match inference_client.execute_task_prompt(&prompt, &TaskId::new("project_generation")).await {
            Ok((response, status)) => {
                println!("Generation Status: {:?}", status);
                println!("AI-Generated Project Configuration:\n{}", response);

                // Optional: Stream additional details
                println!("\nStreaming Additional Details:");
                let stream_response = inference_client.stream_completion(
                    &format!("Provide more implementation details for: {}", prompt_text), 
                    0.7
                ).await?;
                
                println!("Streamed Response:\n{}", stream_response);
            }
            Err(e) => {
                eprintln!("Project generation failed: {}", e);
            }
        }

        // Add a small delay between generations
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    Ok(())
}
