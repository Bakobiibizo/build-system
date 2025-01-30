use std::path::{Path, PathBuf};
use anyhow::Result;
use async_trait::async_trait;
use std::fs;

use crate::doc::types::{Documentation, DocType};
use crate::doc::error::DocumentationError;

pub mod error;
pub mod types;

#[async_trait]
pub trait DocumentationEngine: Send + Sync {
    async fn new(base_path: &Path) -> Self;
    async fn create_doc(&self, doc: &Documentation) -> Result<(), DocumentationError>;
    async fn read_doc(&self, path: &Path) -> Result<Documentation, DocumentationError>;
    async fn update_doc(&self, doc: Documentation) -> Result<(), DocumentationError>;
    async fn delete_doc(&self, path: &Path) -> Result<(), DocumentationError>;
    async fn save_doc(&self, doc: &Documentation) -> Result<(), DocumentationError>;
}

#[derive(Debug, Clone, Default)]
pub struct FileDocumentationEngine {
    pub base_path: PathBuf,
}

impl FileDocumentationEngine {
    pub async fn try_new(base_path: &Path) -> Result<Self, DocumentationError> {
        // Optional: Add any initialization logic here
        Ok(Self {
            base_path: base_path.to_path_buf(),
        })
    }

    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    pub async fn generate_markdown(&self, doc: &Documentation) -> Result<String, DocumentationError> {
        let md_content = match doc.doc_type {
            DocType::ProjectOverview => {
                format!(
                    "# {}\n\n## Description\n{}\n\n## Technologies\n{}\n",
                    doc.title,
                    doc.content,
                    doc.metadata.get("technologies").cloned().unwrap_or_default()
                )
            },
            DocType::Architecture => {
                format!(
                    "# Architecture: {}\n\n## Overview\n{}\n\n## Design Principles\n{}\n",
                    doc.title,
                    doc.content,
                    doc.metadata.get("design_principles").cloned().unwrap_or_default()
                )
            },
            _ => doc.content.clone(),
        };

        Ok(md_content)
    }
}

#[async_trait]
impl DocumentationEngine for FileDocumentationEngine {
    async fn new(base_path: &Path) -> Self {
        Self {
            base_path: base_path.to_path_buf(),
        }
    }

    async fn create_doc(&self, doc: &Documentation) -> Result<(), DocumentationError> {
        // Ensure the directory exists
        fs::create_dir_all(doc.path.parent().unwrap_or(Path::new(".")))?;

        // Generate markdown content
        let markdown_content = self.generate_markdown(doc).await?;

        // Write to file
        fs::write(&doc.path, markdown_content)?;

        Ok(())
    }

    async fn read_doc(&self, path: &Path) -> Result<Documentation, DocumentationError> {
        // Read the markdown content
        let content = fs::read_to_string(path)?;

        // TODO: Implement proper parsing of markdown to Documentation
        Ok(Documentation {
            path: path.to_path_buf(),
            content,
            ..Default::default()
        })
    }

    async fn update_doc(&self, doc: Documentation) -> Result<(), DocumentationError> {
        // Regenerate markdown and write to file
        let markdown_content = self.generate_markdown(&doc).await?;
        fs::write(&doc.path, markdown_content)?;

        Ok(())
    }

    async fn delete_doc(&self, path: &Path) -> Result<(), DocumentationError> {
        if !path.exists() {
            return Err(DocumentationError::DocumentNotFound);
        }

        fs::remove_file(path)?;

        Ok(())
    }

    async fn save_doc(&self, doc: &Documentation) -> Result<(), DocumentationError> {
        let path = &doc.path;
        
        let md_content = self.generate_markdown(doc).await?;
        fs::write(path, md_content)?;

        Ok(())
    }
}
