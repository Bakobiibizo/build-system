// Core modules
pub mod cli;
pub mod doc;
pub mod inference;
pub mod prompt;
pub mod project_generator;
pub mod state;
pub mod build;

// Utility and support modules
pub mod config;
pub mod logging;
pub mod tools;

// AI and inference modules
pub mod llm;

// Optional feature modules
#[cfg(feature = "web-features")]
pub mod web;

#[cfg(feature = "ai-features")]
pub mod ai;

pub mod error;

// Exports
pub use cli::Cli;
pub use cli::Commands;
pub use doc::FileDocumentationEngine;
pub use doc::types::Documentation;
pub use doc::types::DocType;
pub use prompt::PromptManager;
pub use prompt::ProjectConfig;
pub use project_generator::ProjectGenerationWorkflow;
pub use inference::InferenceClient;

pub use cli::run_cli;

#[cfg(test)]
mod inference_test;
