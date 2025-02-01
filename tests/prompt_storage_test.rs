use anyhow::Result;
use build_system::prompt::storage::PromptStorage;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tempfile::tempdir;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct TestPrompt {
    name: String,
    description: String,
    complexity: u8,
}

#[test]
fn test_prompt_storage_full_workflow() -> Result<()> {
    // Create a temporary directory for storage
    let dir = tempdir()?;
    let storage = PromptStorage::new(dir.path())?;

    // JSON Schema for validation
    let schema = json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "description": { "type": "string" },
            "complexity": { 
                "type": "integer", 
                "minimum": 0, 
                "maximum": 10 
            }
        },
        "required": ["name", "description", "complexity"]
    });

    // Create test prompts
    let prompts = vec![
        TestPrompt {
            name: "Web App Prompt".to_string(),
            description: "Generate a modern web application".to_string(),
            complexity: 7,
        },
        TestPrompt {
            name: "Data Science Prompt".to_string(),
            description: "Create a data science project workflow".to_string(),
            complexity: 9,
        }
    ];

    // Store prompts and collect their IDs
    let mut prompt_ids = Vec::new();
    for prompt in &prompts {
        // Validate before storing
        let prompt_value = serde_json::to_value(prompt)?;
        PromptStorage::validate_json(&schema, &prompt_value)?;
        
        // Store the prompt
        let id = storage.store("prompt", prompt)?;
        prompt_ids.push(id);
    }

    // Retrieve and verify prompts
    for (id, original_prompt) in prompt_ids.iter().zip(prompts.iter()) {
        let retrieved_prompt: TestPrompt = storage.retrieve("prompt", id)?;
        assert_eq!(&retrieved_prompt, original_prompt);
    }

    // List prompts
    let listed_prompts = storage.list::<TestPrompt>("prompt")?;
    assert_eq!(listed_prompts.len(), prompts.len());

    // Validate JSON schema with invalid data
    let invalid_prompt = json!({
        "name": 123,  // Invalid type
        "description": "Test",
        "complexity": 15  // Out of range
    });
    
    let validation_result = PromptStorage::validate_json(&schema, &invalid_prompt);
    assert!(validation_result.is_err(), "Invalid JSON should fail validation");

    // Delete a prompt
    storage.delete("prompt", &prompt_ids[0])?;
    let remaining_prompts = storage.list::<TestPrompt>("prompt")?;
    assert_eq!(remaining_prompts.len(), prompts.len() - 1);

    // Flush changes
    storage.flush()?;

    Ok(())
}

#[test]
fn test_prompt_storage_error_cases() -> Result<()> {
    let dir = tempdir()?;
    let storage = PromptStorage::new(dir.path())?;

    // Try to retrieve non-existent prompt
    let non_existent_id = Uuid::new_v4();
    let retrieve_result: Result<TestPrompt> = storage.retrieve("prompt", &non_existent_id);
    assert!(retrieve_result.is_err(), "Retrieving non-existent prompt should fail");

    // Try to delete non-existent prompt
    let delete_result = storage.delete("prompt", &non_existent_id);
    assert!(delete_result.is_ok(), "Deleting non-existent prompt should not error");

    Ok(())
}

#[test]
fn test_prompt_storage_validation() -> Result<()> {
    // Create a schema
    let schema = json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "description": { "type": "string" },
            "complexity": { 
                "type": "integer", 
                "minimum": 0, 
                "maximum": 10 
            }
        },
        "required": ["name", "description", "complexity"]
    });

    // Valid data
    let valid_data = json!({
        "name": "Web App Prompt",
        "description": "Generate a modern web application",
        "complexity": 7
    });

    // Invalid data
    let invalid_data = json!({
        "name": 123,  // Invalid type
        "description": "Test",
        "complexity": 15  // Out of range
    });

    // Validate valid data
    PromptStorage::validate_json(&schema, &valid_data)?;

    // Validate invalid data (should fail)
    let validation_result = PromptStorage::validate_json(&schema, &invalid_data);
    assert!(validation_result.is_err(), "Invalid JSON should fail validation");

    Ok(())
}
