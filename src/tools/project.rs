use anyhow::{Result, anyhow};
use clap::Parser;
use crate::inference::InferenceClient;
use crate::project_generator::{ProjectGenerator, parse_project_design};
use serde_json;

#[derive(Parser, Debug)]
pub struct ProjectArgs {
    /// Name of the project
    #[clap(long)]
    name: String,

    /// Programming language to use
    #[clap(long)]
    language: String,
}

pub async fn handle_project(args: ProjectArgs) -> Result<()> {
    println!("Initializing inference client...");
    let client = InferenceClient::new()?;

    // Create the user request
    let request = format!(
        "Create a {} project named '{}'",
        args.language, args.name
    );
    println!("Sending request: {}", request);

    // Generate project configuration using AI
    let config_json = client.generate_project_config(&request).await?;
    println!("Generated config (raw):\n{}", config_json);

    // Try to parse it as a Value first to check structure
    let parsed = serde_json::from_str::<serde_json::Value>(&config_json)
        .map_err(|e| anyhow!("Invalid JSON: {}", e))?;
    println!("Parsed JSON structure:\n{}", serde_json::to_string_pretty(&parsed)?);

    // Print specific fields we care about
    if let Some(dir_struct) = parsed.get("directory_structure") {
        println!("\nDirectory structure type: {}", dir_struct.is_object());
        if let Some(obj) = dir_struct.as_object() {
            for (dir, files) in obj {
                println!("Dir '{}' files type: {}", dir, files.is_array());
            }
        }
    }

    // Parse the config into a ProjectDesign
    println!("\nParsing config into ProjectDesign...");
    let design = parse_project_design(&config_json)?;

    // Use the project generator to create the project
    println!("Generating project structure...");
    let generator = ProjectGenerator::new(design);
    generator.generate().await?;

    println!("Project generation complete!");
    Ok(())
}
