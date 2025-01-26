use std::collections::HashMap;
use std::path::{Path, PathBuf};
use chrono::Utc;
use mockall::predicate;
use mockall::mock;

use build_system::doc::types::{Documentation, DocumentationStep, DocumentationStepStatus, DocType};
use build_system::doc::error::DocumentationError;

mock! {
    DocumentationEngine {
        async fn create_doc(&self, doc: Documentation) -> Result<(), DocumentationError>;
        async fn read_doc(&self, path: &Path) -> Result<Documentation, DocumentationError>;
        async fn update_doc(&self, doc: Documentation) -> Result<(), DocumentationError>;
        async fn delete_doc(&self, path: &Path) -> Result<(), DocumentationError>;
        async fn list_docs(&self) -> Result<Vec<Documentation>, DocumentationError>;
        async fn get_doc(&self, path: &Path) -> Result<Documentation, DocumentationError>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_doc_creation() -> Result<(), DocumentationError> {
        let doc = Documentation {
            id: "test-doc-1".to_string(),
            path: PathBuf::from("test/doc.md"),
            doc_type: DocType::Markdown,
            title: "Test Document".to_string(),
            description: Some("Test document description".to_string()),
            content: "# Test Content".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            steps: vec![DocumentationStep {
                id: "test-step-1".to_string(),
                title: "Test Step".to_string(),
                description: Some("Test step description".to_string()),
                status: DocumentationStepStatus::Pending,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                completed_at: None,
                code: None,
                output: None,
            }],
            additional_info: HashMap::new(),
            project: "test-project".to_string(),
            priority: "high".to_string(),
            owner: "test-owner".to_string(),
            tags: vec!["test".to_string()],
        };

        let mut mock = MockDocumentationEngine::new();
        mock.expect_create_doc()
            .with(predicate::eq(doc.clone()))
            .times(1)
            .returning(|_| Ok(()));

        mock.create_doc(doc).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_create_doc() -> Result<(), DocumentationError> {
        let mut mock_doc_engine = MockDocumentationEngine::new();
        let path = PathBuf::from("test/doc.md");
        let doc = Documentation {
            id: "test-doc-1".to_string(),
            path: path.clone(),
            doc_type: DocType::Markdown,
            title: "Test Document".to_string(),
            description: Some("Test document description".to_string()),
            content: "Test documentation content".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            steps: vec![],
            additional_info: HashMap::new(),
            project: "test-project".to_string(),
            priority: "high".to_string(),
            owner: "test-owner".to_string(),
            tags: vec!["test".to_string()],
        };

        mock_doc_engine
            .expect_create_doc()
            .with(predicate::eq(doc.clone()))
            .times(1)
            .returning(|_| Ok(()));

        mock_doc_engine.create_doc(doc).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_doc() -> Result<(), DocumentationError> {
        let mut mock_doc_engine = MockDocumentationEngine::new();
        let path = PathBuf::from("test/doc.md");
        let doc = Documentation {
            id: "test-doc-1".to_string(),
            path: path.clone(),
            doc_type: DocType::Markdown,
            title: "Test Document".to_string(),
            description: Some("Test document description".to_string()),
            content: "Test documentation content".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            steps: vec![],
            additional_info: HashMap::new(),
            project: "test-project".to_string(),
            priority: "high".to_string(),
            owner: "test-owner".to_string(),
            tags: vec!["test".to_string()],
        };

        mock_doc_engine
            .expect_read_doc()
            .withf(|p: &Path| p.to_str() == Some("test/doc.md"))
            .times(1)
            .returning(move |_| Ok(doc.clone()));

        let result = mock_doc_engine.read_doc(path.as_path()).await?;
        assert_eq!(result.id, "test-doc-1");
        Ok(())
    }

    #[tokio::test]
    async fn test_update_doc() -> Result<(), DocumentationError> {
        let mut mock_doc_engine = MockDocumentationEngine::new();
        let path = PathBuf::from("test/doc.md");
        let doc = Documentation {
            id: "test-doc-1".to_string(),
            path: path.clone(),
            doc_type: DocType::Markdown,
            title: "Test Document".to_string(),
            description: Some("Test document description".to_string()),
            content: "Test documentation content".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            steps: vec![],
            additional_info: HashMap::new(),
            project: "test-project".to_string(),
            priority: "high".to_string(),
            owner: "test-owner".to_string(),
            tags: vec!["test".to_string()],
        };

        mock_doc_engine
            .expect_update_doc()
            .with(predicate::eq(doc.clone()))
            .times(1)
            .returning(|_| Ok(()));

        mock_doc_engine.update_doc(doc).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_doc_with_steps() -> Result<(), DocumentationError> {
        let mut mock_doc_engine = MockDocumentationEngine::new();
        let doc = Documentation {
            id: "test-doc-1".to_string(),
            path: PathBuf::from("test/doc.md"),
            doc_type: DocType::Markdown,
            title: "Test Document".to_string(),
            description: Some("Test document description".to_string()),
            content: "Test documentation content".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            steps: vec![DocumentationStep {
                id: "test-step-1".to_string(),
                title: "Test Step".to_string(),
                description: Some("Test step description".to_string()),
                status: DocumentationStepStatus::Pending,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                completed_at: None,
                code: None,
                output: None,
            }],
            additional_info: HashMap::new(),
            project: "test-project".to_string(),
            priority: "high".to_string(),
            owner: "test-owner".to_string(),
            tags: vec!["test".to_string()],
        };

        let doc_clone = doc.clone();
        mock_doc_engine
            .expect_update_doc()
            .with(predicate::eq(doc_clone))
            .times(1)
            .returning(|_| Ok(()));

        mock_doc_engine.update_doc(doc).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_doc_fixed() -> Result<(), DocumentationError> {
        let path = PathBuf::from("test/doc.md");
        let path_ref = path.as_path();
        let doc_type = DocType::Markdown;
        let expected_doc = Documentation {
            id: "test-doc-1".to_string(),
            path: path.clone(),
            doc_type,
            title: "Test Document".to_string(),
            description: Some("Test document description".to_string()),
            content: "Test documentation content".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            steps: vec![DocumentationStep {
                title: "Test Step".to_string(),
                description: Some("Test step description".to_string()),
                code: Some("test code".to_string()),
                output: None,
                status: DocumentationStepStatus::Pending,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                completed_at: None,
                id: "test-step-1".to_string(),
            }],
            additional_info: HashMap::new(),
            project: "test-project".to_string(),
            priority: "high".to_string(),
            owner: "test-owner".to_string(),
            tags: vec!["test".to_string()],
        };

        let mut mock_doc_engine = MockDocumentationEngine::new();
        mock_doc_engine
            .expect_read_doc()
            .withf(|p: &Path| p.to_str() == Some("test/doc.md"))
            .times(1)
            .returning(move |_| Ok(expected_doc.clone()));

        let doc = mock_doc_engine.read_doc(path.as_path()).await?;
        assert_eq!(doc.content, "Test documentation content");
        assert_eq!(doc.steps.len(), 1);
        assert_eq!(doc.steps[0].title, "Test Step");
        Ok(())
    }

    #[tokio::test]
    async fn test_update_doc_new() -> Result<(), DocumentationError> {
        let mut mock_doc_engine = MockDocumentationEngine::new();
        let doc = Documentation {
            path: PathBuf::from("test/doc.md"),
            doc_type: DocType::Markdown,
            content: "Test documentation content".to_string(),
            steps: vec![DocumentationStep {
                id: "test-step-1".to_string(),
                title: "Test Step".to_string(),
                description: Some("Test step description".to_string()),
                code: Some("test code".to_string()),
                output: None,
                status: DocumentationStepStatus::Pending,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                completed_at: None,
            }],
            additional_info: HashMap::new(),
            id: "test-doc-1".to_string(),
            title: "Test Document".to_string(),
            description: Some("Test document description".to_string()),
            project: "test-project".to_string(),
            priority: "high".to_string(),
            owner: "test-owner".to_string(),
            tags: vec!["test".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let doc_clone = doc.clone();
        mock_doc_engine
            .expect_update_doc()
            .with(predicate::eq(doc_clone))
            .returning(|_| Ok(()));

        mock_doc_engine.update_doc(doc).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_doc_new() -> Result<(), DocumentationError> {
        let path = PathBuf::from("test/doc.md");
        let path_ref = path.as_path();
        let doc_type = DocType::Markdown;
        let expected_doc = Documentation {
            id: "test-doc-1".to_string(),
            path: path.clone(),
            doc_type,
            title: "Test Document".to_string(),
            description: Some("Test document description".to_string()),
            content: "Test documentation content".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            steps: vec![DocumentationStep {
                id: "test-step-1".to_string(),
                title: "Test Step".to_string(),
                description: Some("Test step description".to_string()),
                code: Some("test code".to_string()),
                output: None,
                status: DocumentationStepStatus::Pending,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                completed_at: None,
            }],
            additional_info: HashMap::new(),
            project: "test-project".to_string(),
            priority: "high".to_string(),
            owner: "test-owner".to_string(),
            tags: vec!["test".to_string()],
        };

        let mut mock_doc_engine = MockDocumentationEngine::new();
        mock_doc_engine
            .expect_read_doc()
            .withf(|p: &Path| p.to_str() == Some("test/doc.md"))
            .times(1)
            .returning(move |_| Ok(expected_doc.clone()));

        let doc = mock_doc_engine.read_doc(path.as_path()).await?;
        assert_eq!(doc.content, "Test documentation content");
        assert_eq!(doc.steps.len(), 1);
        assert_eq!(doc.steps[0].title, "Test Step");
        Ok(())
    }

    #[tokio::test]
    async fn test_update_doc_with_completed_step() -> Result<(), DocumentationError> {
        let mut mock_doc_engine = MockDocumentationEngine::new();
        let mut doc = Documentation {
            path: PathBuf::from("test/doc.md"),
            doc_type: DocType::Markdown,
            content: "Test documentation content".to_string(),
            steps: vec![DocumentationStep {
                id: "test-step-1".to_string(),
                title: "Test Step".to_string(),
                description: Some("Test step description".to_string()),
                code: Some("test code".to_string()),
                output: Some("test output".to_string()),
                status: DocumentationStepStatus::Completed,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                completed_at: Some(Utc::now()),
            }],
            additional_info: HashMap::new(),
            id: "test-doc-1".to_string(),
            title: "Test Document".to_string(),
            description: Some("Test document description".to_string()),
            project: "test-project".to_string(),
            priority: "high".to_string(),
            owner: "test-owner".to_string(),
            tags: vec!["test".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let doc_clone = doc.clone();
        mock_doc_engine
            .expect_update_doc()
            .with(predicate::eq(doc_clone))
            .returning(|_| Ok(()));

        mock_doc_engine.update_doc(doc).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_doc() -> Result<(), DocumentationError> {
        let mut mock_doc_engine = MockDocumentationEngine::new();
        let path = PathBuf::from("test/doc.md");

        mock_doc_engine
            .expect_delete_doc()
            .withf(|p: &Path| p.to_str() == Some("test/doc.md"))
            .times(1)
            .returning(|_| Ok(()));

        mock_doc_engine.delete_doc(path.as_path()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_list_docs() -> Result<(), DocumentationError> {
        let mut mock_doc_engine = MockDocumentationEngine::new();
        let docs = vec![
            Documentation {
                id: "test-doc-1".to_string(),
                path: PathBuf::from("test/doc1.md"),
                doc_type: DocType::Markdown,
                title: "Test Document 1".to_string(),
                description: Some("Test document 1 description".to_string()),
                content: "Test documentation content 1".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                steps: vec![],
                additional_info: HashMap::new(),
                project: "test-project".to_string(),
                priority: "high".to_string(),
                owner: "test-owner".to_string(),
                tags: vec!["test".to_string()],
            },
            Documentation {
                id: "test-doc-2".to_string(),
                path: PathBuf::from("test/doc2.md"),
                doc_type: DocType::Markdown,
                title: "Test Document 2".to_string(),
                description: Some("Test document 2 description".to_string()),
                content: "Test documentation content 2".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                steps: vec![],
                additional_info: HashMap::new(),
                project: "test-project".to_string(),
                priority: "high".to_string(),
                owner: "test-owner".to_string(),
                tags: vec!["test".to_string()],
            },
        ];

        mock_doc_engine
            .expect_list_docs()
            .times(1)
            .returning(move || Ok(docs.clone()));

        let result = mock_doc_engine.list_docs().await?;
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, "test-doc-1");
        assert_eq!(result[1].id, "test-doc-2");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_doc() -> Result<(), DocumentationError> {
        let mut mock_doc_engine = MockDocumentationEngine::new();
        let path = PathBuf::from("test/doc.md");
        let doc = Documentation {
            id: "test-doc-1".to_string(),
            path: path.clone(),
            doc_type: DocType::Markdown,
            title: "Test Document".to_string(),
            description: Some("Test document description".to_string()),
            content: "Test documentation content".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            steps: vec![],
            additional_info: HashMap::new(),
            project: "test-project".to_string(),
            priority: "high".to_string(),
            owner: "test-owner".to_string(),
            tags: vec!["test".to_string()],
        };

        mock_doc_engine
            .expect_get_doc()
            .withf(|p: &Path| p.to_str() == Some("test/doc.md"))
            .times(1)
            .returning(move |_| Ok(doc.clone()));

        let result = mock_doc_engine.get_doc(path.as_path()).await?;
        assert_eq!(result.id, "test-doc-1");
        assert_eq!(result.title, "Test Document");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_doc_not_found() -> Result<(), DocumentationError> {
        let mut mock_doc_engine = MockDocumentationEngine::new();
        let path = PathBuf::from("test/nonexistent.md");

        mock_doc_engine
            .expect_get_doc()
            .withf(|p: &Path| p.to_str() == Some("test/nonexistent.md"))
            .times(1)
            .returning(|_| Err(DocumentationError::NotFound("Document not found".to_string())));

        let result = mock_doc_engine.get_doc(path.as_path()).await;
        assert!(matches!(result, Err(DocumentationError::NotFound(_))));
        Ok(())
    }
}
