use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use clap::Parser;

mod build;
pub mod project;
pub use project::{ProjectArgs, handle_project};
pub use build::BuildTool;

/// Represents a tool in the system
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

/// Trait for executable tools
#[async_trait::async_trait]
pub trait ExecutableTool: Send + Sync {
    async fn execute(&self, arguments: &str) -> Result<String, String>;
    fn get_tool_definition(&self) -> Tool;
    fn get_short_description(&self) -> String;
    fn get_long_description(&self) -> String;
}

/// Tool registry to manage available tools
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn ExecutableTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };
        
        // Register tools
        registry.register_tool("build".to_string(), Box::new(BuildTool::default()));
        
        registry
    }

    pub fn register_tool<T: ExecutableTool + 'static>(&mut self, name: String, tool: Box<T>) {
        self.tools.insert(name, tool);
    }

    pub async fn execute_tool(&self, tool_call: &ToolCall) -> Result<ToolResult, String> {
        let tool = self.tools.get(&tool_call.name)
            .ok_or_else(|| format!("Tool '{}' not found", tool_call.name))?;
        
        let output = tool.execute(&tool_call.arguments).await?;
        Ok(ToolResult {
            tool_name: tool_call.name.clone(),
            output,
        })
    }

    pub fn get_tool_definitions(&self) -> Vec<Tool> {
        self.tools.values()
            .map(|tool| tool.get_tool_definition())
            .collect()
    }

    pub fn get_tool_descriptions(&self) -> Vec<(String, String)> {
        self.tools.values()
            .map(|tool| {
                let def = tool.get_tool_definition();
                (def.name, tool.get_short_description())
            })
            .collect()
    }

    pub fn get_tool_long_description(&self, name: &str) -> Option<String> {
        self.tools.get(name)
            .map(|tool| tool.get_long_description())
    }
}

impl Debug for ToolRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolRegistry")
            .field("tools", &self.tools.keys().collect::<Vec<_>>())
            .finish()
    }
}

/// Tool call representation
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: String,
}

/// Tool execution result
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_name: String,
    pub output: String,
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

/// Function details for a tool call
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub arguments: String, // JSON string of arguments
}

pub async fn run_tool(tool_name: &str, args: Vec<String>) -> Result<()> {
    match tool_name {
        "project" => {
            let args = ProjectArgs::try_parse_from(args)?;
            handle_project(args).await
        }
        _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name))
    }
}
