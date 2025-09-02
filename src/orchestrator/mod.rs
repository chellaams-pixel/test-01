use crate::{config::Config, upload::UploadManager, workflow::WorkflowEngine};
use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug)]
pub struct AutomationOrchestrator {
    config: Config,
    upload_manager: UploadManager,
    workflow_engine: WorkflowEngine,
    active_tasks: Arc<DashMap<Uuid, TaskInfo>>,
    semaphore: Arc<Semaphore>,
}

#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: Uuid,
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TaskType {
    Upload,
    Workflow,
    System,
}

#[derive(Debug, Clone)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl AutomationOrchestrator {
    pub fn new(
        config: Config,
        upload_manager: UploadManager,
        workflow_engine: WorkflowEngine,
    ) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.workflow.max_concurrent_workflows));
        
        Self {
            config,
            upload_manager,
            workflow_engine,
            active_tasks: Arc::new(DashMap::new()),
            semaphore,
        }
    }

    pub async fn process_upload(&self, upload_path: &str) -> Result<()> {
        let task_id = Uuid::new_v4();
        let task_info = TaskInfo {
            id: task_id,
            task_type: TaskType::Upload,
            status: TaskStatus::Pending,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            error_message: None,
        };

        self.active_tasks.insert(task_id, task_info.clone());
        info!("Starting upload task: {}", task_id);

        let _permit = self.semaphore.acquire().await?;
        
        // Update task status to running
        if let Some(mut task) = self.active_tasks.get_mut(&task_id) {
            task.status = TaskStatus::Running;
            task.started_at = Some(chrono::Utc::now());
        }

        let result = self.upload_manager.process_upload(upload_path).await;

        // Update task status based on result
        if let Some(mut task) = self.active_tasks.get_mut(&task_id) {
            match &result {
                Ok(_) => {
                    task.status = TaskStatus::Completed;
                    info!("Upload task {} completed successfully", task_id);
                }
                Err(e) => {
                    task.status = TaskStatus::Failed;
                    task.error_message = Some(e.to_string());
                    error!("Upload task {} failed: {}", task_id, e);
                }
            }
            task.completed_at = Some(chrono::Utc::now());
        }

        result.map(|_| ())
    }

    pub async fn execute_workflow(&self, workflow_path: &str) -> Result<()> {
        let task_id = Uuid::new_v4();
        let task_info = TaskInfo {
            id: task_id,
            task_type: TaskType::Workflow,
            status: TaskStatus::Pending,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            error_message: None,
        };

        self.active_tasks.insert(task_id, task_info.clone());
        info!("Starting workflow task: {}", task_id);

        let _permit = self.semaphore.acquire().await?;
        
        // Update task status to running
        if let Some(mut task) = self.active_tasks.get_mut(&task_id) {
            task.status = TaskStatus::Running;
            task.started_at = Some(chrono::Utc::now());
        }

        let result = self.workflow_engine.execute_workflow(workflow_path).await;

        // Update task status based on result
        if let Some(mut task) = self.active_tasks.get_mut(&task_id) {
            match &result {
                Ok(_) => {
                    task.status = TaskStatus::Completed;
                    info!("Workflow task {} completed successfully", task_id);
                }
                Err(e) => {
                    task.status = TaskStatus::Failed;
                    task.error_message = Some(e.to_string());
                    error!("Workflow task {} failed: {}", task_id, e);
                }
            }
            task.completed_at = Some(chrono::Utc::now());
        }

        result.map(|_| ())
    }

    pub fn get_task_status(&self, task_id: Uuid) -> Option<TaskInfo> {
        self.active_tasks.get(&task_id).map(|task| task.clone())
    }

    pub fn list_active_tasks(&self) -> Vec<TaskInfo> {
        self.active_tasks
            .iter()
            .map(|task| task.clone())
            .collect()
    }

    pub async fn cancel_task(&self, task_id: Uuid) -> Result<()> {
        if let Some(mut task) = self.active_tasks.get_mut(&task_id) {
            task.status = TaskStatus::Cancelled;
            task.completed_at = Some(chrono::Utc::now());
            info!("Task {} cancelled", task_id);
        } else {
            warn!("Task {} not found", task_id);
        }
        Ok(())
    }

    pub async fn cleanup_completed_tasks(&self) {
        let mut to_remove = Vec::new();
        
        for task in self.active_tasks.iter() {
            match task.status {
                TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled => {
                    // Remove tasks older than 24 hours
                    if let Some(completed_at) = task.completed_at {
                        let duration = chrono::Utc::now() - completed_at;
                        if duration.num_hours() > 24 {
                            to_remove.push(task.id);
                        }
                    }
                }
                _ => {}
            }
        }

        let removed_count = to_remove.len();
        for task_id in to_remove {
            self.active_tasks.remove(&task_id);
        }

        info!("Cleaned up {} completed tasks", removed_count);
    }
}
