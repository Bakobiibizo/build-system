use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a tool in the system
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: ToolParameters,
}

/// JSON Schema-like parameter definition for tools
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolParameters {
    #[serde(rename = "type")]
    pub param_type: String,
    pub properties: HashMap<String, ParameterDefinition>,
    pub required: Vec<String>,
}

/// Individual parameter definition
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParameterDefinition {
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: Option<String>,
    pub enum_values: Option<Vec<String>>,
}

/// Tool call representation
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: ToolFunction,
}

/// Function details for a tool call
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub arguments: String, // JSON string of arguments
}

/// Tool execution result
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub output: String,
}

/// Trait for executable tools
pub trait ExecutableTool {
    fn execute(&self, arguments: &str) -> Result<String, String>;
}

/// Tool registry to manage available tools
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn ExecutableTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        ToolRegistry {
            tools: HashMap::new(),
        }
    }

    pub fn register_tool<T: ExecutableTool + 'static>(&mut self, name: String, tool: T) {
        self.tools.insert(name, Box::new(tool));
    }

    pub fn execute_tool(&self, tool_call: &ToolCall) -> Result<ToolResult, String> {
        let tool = self.tools.get(&tool_call.function.name)
            .ok_or_else(|| format!("Tool '{}' not found", tool_call.function.name))?;

        let output = tool.execute(&tool_call.function.arguments)?;

        Ok(ToolResult {
            tool_call_id: tool_call.id.clone(),
            output,
        })
    }

    pub fn get_tool_definitions(&self) -> Vec<Tool> {
        // This would be implemented to return registered tool definitions
        vec![]
    }
}
