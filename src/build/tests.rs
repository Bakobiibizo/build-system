#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;
    use anyhow::Result;
    use tokio::time::sleep;
    use crate::build::{
        BuildEngine, BuildError, BuildTask, BuildStep, ResourceRequirements,
        BuildMetadata, BuildPriority, TaskStatus, BuildExecutor,
    };
    use crate::state::StateManager;
    use crate::doc::DocumentationEngine;
    use mockall::*;
    use async_trait::async_trait;

    mock! {
        pub BuildExecutor {}

        #[async_trait]
        impl BuildExecutor for MockBuildExecutor {
            async fn execute_task(&self, task: BuildTask) -> Result<(), BuildError>;
            async fn cancel_task(&self, task_id: &str) -> Result<(), BuildError>;
            async fn get_task_status(&self, task_id: &str) -> Result<TaskStatus, BuildError>;
        }

        impl Clone for MockBuildExecutor {
            fn clone(&self) -> Self {
                Self::default()
            }
        }
    }

    mock! {
        pub StateManager {}

        impl StateManager {
            fn update_task_status(&self, id: String, new_status: TaskStatus) -> Result<()>;
        }
    }

    mock! {
        pub DocumentationEngine {}

        impl DocumentationEngine {
            fn update_doc(&self, task: BuildTask) -> Result<()>;
        }
    }

    fn create_test_task() -> BuildTask {
        BuildTask {
            id: "test-task-1".to_string(),
            steps: vec![
                BuildStep {
                    id: "step-1".to_string(),
                    command: "echo".to_string(),
                    args: vec!["test".to_string()],
                    env: Default::default(),
                    working_dir: Default::default(),
                    timeout: Duration::from_secs(30),
                    dependencies: vec![],
                }
            ],
            resources: ResourceRequirements {
                cpu_cores: 1,
                memory_mb: 1024,
                disk_mb: 1024,
                network_access: false,
            },
            timeout: Duration::from_secs(60),
            metadata: BuildMetadata {
                owner: "test".to_string(),
                project: "test-project".to_string(),
                priority: BuildPriority::Normal,
                estimated_duration: Duration::from_secs(30),
                tags: vec!["test".to_string()],
            },
        }
    }

    #[tokio::test]
    async fn test_execute_task_success() {
        let mut mock_executor = MockBuildExecutor::new();
        mock_executor
            .expect_execute_task()
            .returning(|_| Ok(()));

        let mut mock_state_manager = MockStateManager::new();
        mock_state_manager
            .expect_update_task_status()
            .returning(|_, _| Ok(()));

        let mut mock_doc_engine = MockDocumentationEngine::new();
        mock_doc_engine
            .expect_update_doc()
            .returning(|_| Ok(()));

        let engine = BuildEngine::new(
            mock_state_manager,
            mock_doc_engine,
            mock_executor,
            2,
        );

        let task = create_test_task();
        let result = engine.execute_task(task).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_task_failure() {
        let mut mock_executor = MockBuildExecutor::new();
        mock_executor
            .expect_execute_task()
            .returning(|_| Err(BuildError::ExecutionFailed("Test failure".to_string())));

        let mut mock_state_manager = MockStateManager::new();
        mock_state_manager
            .expect_update_task_status()
            .returning(|_, _| Ok(()));

        let mut mock_doc_engine = MockDocumentationEngine::new();
        mock_doc_engine
            .expect_update_doc()
            .returning(|_| Ok(()));

        let engine = BuildEngine::new(
            mock_state_manager,
            mock_doc_engine,
            mock_executor,
            2,
        );

        let task = create_test_task();
        let result = engine.execute_task(task).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BuildError::ExecutionFailed(_)));
    }

    #[tokio::test]
    async fn test_cancel_task() {
        let mut mock_executor = MockBuildExecutor::new();
        mock_executor
            .expect_execute_task()
            .returning(|_| {
                sleep(Duration::from_secs(1));
                Ok(())
            });
        mock_executor
            .expect_cancel_task()
            .returning(|_| Ok(()));

        let mut mock_state_manager = MockStateManager::new();
        mock_state_manager
            .expect_update_task_status()
            .returning(|_, _| Ok(()));

        let mut mock_doc_engine = MockDocumentationEngine::new();
        mock_doc_engine
            .expect_update_doc()
            .returning(|_| Ok(()));

        let engine = BuildEngine::new(
            mock_state_manager,
            mock_doc_engine,
            mock_executor,
            2,
        );

        let task = create_test_task();
        let task_id = task.id.clone();

        // Start task execution
        let _handle = tokio::spawn(async move {
            engine.execute_task(task).await
        });

        // Give some time for the task to start
        sleep(Duration::from_millis(100)).await;

        // Cancel the task
        let result = engine.cancel_task(&task_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_tasks() {
        let mut mock_executor = MockBuildExecutor::new();
        mock_executor
            .expect_execute_task()
            .times(2)
            .returning(|_| Ok(()));

        let mut mock_state_manager = MockStateManager::new();
        mock_state_manager
            .expect_update_task_status()
            .returning(|_, _| Ok(()));

        let mut mock_doc_engine = MockDocumentationEngine::new();
        mock_doc_engine
            .expect_update_doc()
            .returning(|_| Ok(()));

        let engine = BuildEngine::new(
            mock_state_manager,
            mock_doc_engine,
            mock_executor,
            2,
        );

        let mut task1 = create_test_task();
        task1.id = "task-1".to_string();
        let mut task2 = create_test_task();
        task2.id = "task-2".to_string();

        // Execute tasks with timeout
        let timeout = Duration::from_millis(100);
        let result1 = tokio::time::timeout(timeout, engine.execute_task(task1)).await;
        let result2 = tokio::time::timeout(timeout, engine.execute_task(task2)).await;

        assert!(result1.is_ok() && result1.unwrap().is_ok());
        assert!(result2.is_ok() && result2.unwrap().is_ok());
    }
}
