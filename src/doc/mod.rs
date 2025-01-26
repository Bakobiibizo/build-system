use std::path::{Path, PathBuf};
use async_trait::async_trait;
use chrono::Utc;
use serde_json;
use std::collections::HashMap;

use crate::doc::error::DocumentationError;
use crate::doc::types::{Documentation, DocType};

pub mod error;
pub mod types;

#[async_trait]
pub trait DocumentationEngine: Send + Sync {
    async fn create_doc(&self, doc: Documentation) -> Result<(), DocumentationError>;
    async fn read_doc(&self, path: &Path) -> Result<Documentation, DocumentationError>;
    async fn update_doc(&self, doc: Documentation) -> Result<(), DocumentationError>;
    async fn delete_doc(&self, path: &Path) -> Result<(), DocumentationError>;
}

pub struct FileDocumentationEngine {
    base_path: PathBuf,
}

impl FileDocumentationEngine {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    fn get_file_path(&self, doc: &Documentation) -> PathBuf {
        self.base_path.join(format!("{}.{}", doc.id, self.get_file_extension(&doc.doc_type)))
    }

    fn get_file_extension(&self, doc_type: &DocType) -> &'static str {
        match doc_type {
            DocType::Markdown => "md",
            DocType::Html => "html",
            DocType::Text => "txt",
            DocType::Json => "json",
        }
    }

    async fn write_doc(&self, doc: &Documentation, path: &Path) -> Result<(), DocumentationError> {
        let mut md_content = String::new();

        md_content.push_str(&format!("# {}\n\n", doc.title));
        
        if let Some(desc) = &doc.description {
            md_content.push_str(&format!("## Description\n\n{}\n\n", desc));
        }

        md_content.push_str("## Content\n\n");
        md_content.push_str(&doc.content);
        md_content.push_str("\n\n");

        md_content.push_str("## Steps\n\n");
        for step in &doc.steps {
            md_content.push_str(&format!("### {}\n\n", step.title));
            if let Some(desc) = &step.description {
                md_content.push_str(&format!("{}\n\n", desc));
            }
            md_content.push_str(&format!("Status: {}\n\n", step.status));
        }

        tokio::fs::write(path, md_content).await
            .map_err(|e| DocumentationError::IoError(e))
    }
}

#[async_trait]
impl DocumentationEngine for FileDocumentationEngine {
    async fn create_doc(&self, doc: Documentation) -> Result<(), DocumentationError> {
        let path = self.get_file_path(&doc);
        if path.exists() {
            return Err(DocumentationError::AlreadyExists(path.to_string_lossy().into()));
        }

        match doc.doc_type {
            DocType::Markdown => {
                self.write_doc(&doc, &path).await?;
            }
            DocType::Html => {
                // TODO: Implement HTML generation
                unimplemented!()
            }
            DocType::Text => {
                tokio::fs::write(&path, doc.content).await
                    .map_err(|e| DocumentationError::IoError(e))?;
            }
            DocType::Json => {
                let json = serde_json::to_string_pretty(&doc)
                    .map_err(|e| DocumentationError::SerializationError(e.to_string()))?;
                tokio::fs::write(&path, json).await
                    .map_err(|e| DocumentationError::IoError(e))?;
            }
        }

        Ok(())
    }

    async fn read_doc(&self, path: &Path) -> Result<Documentation, DocumentationError> {
        if !path.exists() {
            return Err(DocumentationError::NotFound(path.to_string_lossy().into()));
        }

        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| DocumentationError::IoError(e))?;

        match path.extension().and_then(|ext| ext.to_str()) {
            Some("md") => {
                let now = Utc::now();
                Ok(Documentation {
                    id: path.file_stem().unwrap().to_string_lossy().into(),
                    title: "Untitled".to_string(),
                    description: None,
                    content,
                    doc_type: DocType::Markdown,
                    path: path.to_path_buf(),
                    project: String::new(),
                    owner: String::new(),
                    priority: String::new(),
                    tags: vec![],
                    additional_info: HashMap::new(),
                    steps: vec![],
                    created_at: now,
                    updated_at: now,
                })
            }
            Some("json") => {
                serde_json::from_str(&content)
                    .map_err(|e| DocumentationError::DeserializationError(e.to_string()))
            }
            _ => Err(DocumentationError::InvalidDocType(
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
            )),
        }
    }

    async fn update_doc(&self, doc: Documentation) -> Result<(), DocumentationError> {
        let path = self.get_file_path(&doc);
        if !path.exists() {
            return Err(DocumentationError::NotFound(path.to_string_lossy().into()));
        }

        match doc.doc_type {
            DocType::Markdown => {
                self.write_doc(&doc, &path).await?;
            }
            DocType::Html => {
                // TODO: Implement HTML generation
                unimplemented!()
            }
            DocType::Text => {
                tokio::fs::write(&path, doc.content).await
                    .map_err(|e| DocumentationError::IoError(e))?;
            }
            DocType::Json => {
                let json = serde_json::to_string_pretty(&doc)
                    .map_err(|e| DocumentationError::SerializationError(e.to_string()))?;
                tokio::fs::write(&path, json).await
                    .map_err(|e| DocumentationError::IoError(e))?;
            }
        }

        Ok(())
    }

    async fn delete_doc(&self, path: &Path) -> Result<(), DocumentationError> {
        if !path.exists() {
            return Err(DocumentationError::NotFound(path.to_string_lossy().into()));
        }

        tokio::fs::remove_file(path).await
            .map_err(|e| DocumentationError::IoError(e))
    }
}
