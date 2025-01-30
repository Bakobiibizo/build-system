use build_system::tools::{ToolRegistry, ToolCall};
use build_system::project_generator::{parse_project_design};
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create tool registry
    let mut registry = ToolRegistry::new();

    // Sample project design
    let design_json = r#"{
        "project_name": "openai-streaming-client",
        "system_architecture": "Modular Python client for OpenAI API with streaming support",
        "design_principles": [
            "Separation of Concerns",
            "Error Resilience",
            "Configurability"
        ]
    }"#;

    // Parse project design
    let project_design = parse_project_design(design_json)?;

    // Register project generator tool
    registry.register_tool(
        "generate_project".to_string(), 
        project_design
    );

    // Simulate a tool call
    let tool_call = ToolCall {
        id: Uuid::new_v4().to_string(),
        call_type: "function".to_string(),
        function: build_system::tools::ToolFunction {
            name: "generate_project".to_string(),
            arguments: r#"{"project_name": "openai-streaming-client", "language": "python"}"#.to_string(),
        },
    };

    // Execute the tool
    match registry.execute_tool(&tool_call) {
        Ok(result) => {
            println!("Tool execution result: {}", result.output);
            Ok(())
        },
        Err(e) => {
            eprintln!("Tool execution failed: {}", e);
            Err(e.into())
        }
    }
}
