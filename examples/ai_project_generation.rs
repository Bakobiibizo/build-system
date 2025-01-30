use anyhow::Result;
use build_system::inference::InferenceClient;
use build_system::prompt::{PromptGenerator, ProjectConfig};
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

        // Create a sample project configuration
        let project_config = ProjectConfig::new("MyProject")
            .with_description(prompt_text)
            .with_project_type(build_system::prompt::ProjectType::WebApplication);

        // Generate a prompt for project generation
        let prompt = PromptGenerator::generate_project_prompt(&project_config);

        // Execute AI-powered project generation
        match inference_client.execute_task_prompt(&prompt, &TaskId::new("project_generation")).await {
            Ok(response) => {
                println!("Generation Status: Success");
                println!("AI-Generated Project Configuration:\n{}", response);
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
