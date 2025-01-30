use build_system::project_generator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sample project design JSON 
    let design_json = r#"{
        "project_name": "openai-streaming-client",
        "system_architecture": "Modular Python client for OpenAI API with streaming support",
        "design_principles": [
            "Separation of Concerns",
            "Error Resilience",
            "Configurability"
        ],
        "component_responsibilities": {
            "OpenAIClient": "Manage API interactions",
            "ErrorHandler": "Implement retry and error management",
            "LoggingManager": "Configure and manage logging"
        },
        "error_handling": {
            "timeout": 30,
            "retry": 3,
            "error_types": ["NetworkError", "RateLimitError"]
        },
        "performance_scalability": {
            "caching": "In-memory LRU cache",
            "connection_pooling": "Enabled",
            "load_balancing": "Client-side round-robin"
        },
        "logging_monitoring": {
            "log_levels": ["INFO", "WARNING", "ERROR"],
            "log_storage": "Console and File",
            "monitoring_tools": ["Prometheus"]
        },
        "configuration_management": {
            "config_file_format": ".env",
            "environment_variables": ["OPENAI_API_KEY", "LOG_LEVEL"],
            "config_loading": "python-dotenv"
        },
        "recommendations": [
            "Use environment variables for sensitive configurations",
            "Implement comprehensive error handling",
            "Monitor API usage and performance"
        ]
    }"#;

    // Generate project structure
    project_generator::generate_project(design_json)?;

    println!("Project 'openai-streaming-client' generated successfully!");
    Ok(())
}
