use crate::config::WorkflowConfig;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use tracing::{error, info};
use uuid::Uuid;

#[derive(Debug)]
pub struct WorkflowEngine {
    config: WorkflowConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub steps: Vec<WorkflowStep>,
    pub variables: HashMap<String, String>,
    pub metadata: WorkflowMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub step_type: StepType,
    pub command: String,
    pub args: Vec<String>,
    pub timeout: Option<u64>,
    pub retry_count: Option<u32>,
    pub depends_on: Vec<String>,
    pub condition: Option<String>,
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    Command,
    Script,
    Upload,
    Download,
    Transform,
    Validate,
    Notify,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub author: String,
    pub tags: Vec<String>,
    pub priority: WorkflowPriority,
    pub estimated_duration: Option<u64>,
    pub resource_requirements: ResourceRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: u32,
    pub memory_mb: u32,
    pub disk_space_mb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub steps_executed: Vec<StepExecution>,
    pub variables: HashMap<String, String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecution {
    pub step_id: String,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub output: Option<String>,
    pub error_message: Option<String>,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Skipped,
}

impl WorkflowEngine {
    pub fn new(config: WorkflowConfig) -> Self {
        Self { config }
    }

    pub async fn execute_workflow(&self, workflow_path: &str) -> Result<WorkflowExecution> {
        let workflow = self.load_workflow(workflow_path).await?;
        let execution_id = Uuid::new_v4();
        
        info!("Starting workflow execution {}: {}", execution_id, workflow.name);

        let mut execution = WorkflowExecution {
            id: execution_id,
            workflow_id: workflow.id,
            status: ExecutionStatus::Pending,
            started_at: Utc::now(),
            completed_at: None,
            steps_executed: Vec::new(),
            variables: workflow.variables.clone(),
            error_message: None,
        };

        // Execute workflow steps
        self.execute_workflow_steps(&workflow, &mut execution).await?;

        execution.completed_at = Some(Utc::now());
        execution.status = ExecutionStatus::Completed;

        // Save execution record
        self.save_execution_record(&execution).await?;

        info!("Workflow execution {} completed successfully", execution_id);
        Ok(execution)
    }

    async fn load_workflow(&self, workflow_path: &str) -> Result<Workflow> {
        let path = Path::new(workflow_path);
        
        if !path.exists() {
            return Err(anyhow::anyhow!("Workflow file does not exist: {}", workflow_path));
        }

        let content = fs::read_to_string(path)?;
        let workflow: Workflow = serde_json::from_str(&content)?;
        
        info!("Loaded workflow: {} (version: {})", workflow.name, workflow.version);
        Ok(workflow)
    }

    async fn execute_workflow_steps(
        &self,
        workflow: &Workflow,
        execution: &mut WorkflowExecution,
    ) -> Result<()> {
        execution.status = ExecutionStatus::Running;

        // Sort steps by dependencies
        let sorted_steps = self.sort_steps_by_dependencies(&workflow.steps)?;

        for step in sorted_steps {
            let step_execution = self.execute_step(step, execution).await?;
            execution.steps_executed.push(step_execution);

            // Check if any step failed
            if let Some(failed_step) = execution.steps_executed.iter().find(|s| {
                matches!(s.status, ExecutionStatus::Failed)
            }) {
                execution.status = ExecutionStatus::Failed;
                execution.error_message = failed_step.error_message.clone();
                return Err(anyhow::anyhow!("Step {} failed: {:?}", 
                    failed_step.step_id, failed_step.error_message));
            }
        }

        Ok(())
    }

    fn sort_steps_by_dependencies<'a>(&self, steps: &'a [WorkflowStep]) -> Result<Vec<&'a WorkflowStep>> {
        let mut sorted = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut visiting = std::collections::HashSet::new();

        for step in steps {
            if !visited.contains(&step.id) {
                self.visit_step(step, steps, &mut visited, &mut visiting, &mut sorted)?;
            }
        }

        Ok(sorted)
    }

    fn visit_step<'a>(
        &self,
        step: &'a WorkflowStep,
        all_steps: &'a [WorkflowStep],
        visited: &mut std::collections::HashSet<String>,
        visiting: &mut std::collections::HashSet<String>,
        sorted: &mut Vec<&'a WorkflowStep>,
    ) -> Result<()> {
        if visiting.contains(&step.id) {
            return Err(anyhow::anyhow!("Circular dependency detected for step: {}", step.id));
        }

        if visited.contains(&step.id) {
            return Ok(());
        }

        visiting.insert(step.id.clone());

        for dep_id in &step.depends_on {
            let dep_step = all_steps.iter().find(|s| &s.id == dep_id)
                .ok_or_else(|| anyhow::anyhow!("Dependency step not found: {}", dep_id))?;
            self.visit_step(dep_step, all_steps, visited, visiting, sorted)?;
        }

        visiting.remove(&step.id);
        visited.insert(step.id.clone());
        sorted.push(step);

        Ok(())
    }

    async fn execute_step(
        &self,
        step: &WorkflowStep,
        execution: &WorkflowExecution,
    ) -> Result<StepExecution> {
        let mut step_execution = StepExecution {
            step_id: step.id.clone(),
            status: ExecutionStatus::Pending,
            started_at: Utc::now(),
            completed_at: None,
            output: None,
            error_message: None,
            retry_count: 0,
        };

        info!("Executing step: {} ({})", step.name, step.id);

        // Check if step should be skipped based on condition
        if let Some(condition) = &step.condition {
            if !self.evaluate_condition(condition, execution).await? {
                step_execution.status = ExecutionStatus::Skipped;
                step_execution.completed_at = Some(Utc::now());
                info!("Step {} skipped due to condition", step.id);
                return Ok(step_execution);
            }
        }

        step_execution.status = ExecutionStatus::Running;

        let max_retries = step.retry_count.unwrap_or(self.config.retry_attempts);
        let mut last_error = None;

        for attempt in 0..=max_retries {
            if attempt > 0 {
                step_execution.retry_count = attempt;
                info!("Retrying step {} (attempt {}/{})", step.id, attempt, max_retries);
            }

            match self.execute_step_command(step, execution).await {
                Ok(output) => {
                    step_execution.output = Some(output);
                    step_execution.status = ExecutionStatus::Completed;
                    step_execution.completed_at = Some(Utc::now());
                    info!("Step {} completed successfully", step.id);
                    return Ok(step_execution);
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                    step_execution.error_message = Some(e.to_string());
                    
                    if attempt < max_retries {
                        // Wait before retry
                        tokio::time::sleep(tokio::time::Duration::from_secs(2_u64.pow(attempt as u32))).await;
                    }
                }
            }
        }

        step_execution.status = ExecutionStatus::Failed;
        step_execution.completed_at = Some(Utc::now());
        error!("Step {} failed after {} attempts: {:?}", step.id, max_retries, last_error);

        Ok(step_execution)
    }

    async fn execute_step_command(
        &self,
        step: &WorkflowStep,
        execution: &WorkflowExecution,
    ) -> Result<String> {
        let timeout = step.timeout.unwrap_or(self.config.timeout_seconds);
        
        let output = tokio::time::timeout(
            tokio::time::Duration::from_secs(timeout),
            self.run_command(&step.command, &step.args, execution)
        ).await??;

        Ok(output)
    }

    async fn run_command(
        &self,
        command: &str,
        args: &[String],
        execution: &WorkflowExecution,
    ) -> Result<String> {
        let mut cmd = Command::new(command);
        cmd.args(args);

        // Set environment variables from workflow execution
        for (key, value) in &execution.variables {
            cmd.env(key, value);
        }

        let output = cmd.output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Command failed: {}", stderr))
        }
    }

    async fn evaluate_condition(&self, condition: &str, execution: &WorkflowExecution) -> Result<bool> {
        // Simple condition evaluation - can be extended with a proper expression parser
        if condition.contains("$") {
            // Replace variables with their values
            let mut evaluated_condition = condition.to_string();
            for (key, value) in &execution.variables {
                let placeholder = format!("${}", key);
                evaluated_condition = evaluated_condition.replace(&placeholder, value);
            }
            
            // Simple boolean evaluation
            Ok(evaluated_condition.to_lowercase() == "true")
        } else {
            Ok(true)
        }
    }

    async fn save_execution_record(&self, execution: &WorkflowExecution) -> Result<()> {
        let executions_dir = self.config.workflow_dir.join("executions");
        fs::create_dir_all(&executions_dir)?;
        
        let record_path = executions_dir.join(format!("{}.json", execution.id));
        let record_json = serde_json::to_string_pretty(execution)?;
        fs::write(record_path, record_json)?;

        info!("Workflow execution record saved: {}", execution.id);
        Ok(())
    }

    pub async fn list_workflows(&self) -> Result<Vec<Workflow>> {
        let mut workflows = Vec::new();

        if self.config.workflow_dir.exists() {
            for entry in fs::read_dir(&self.config.workflow_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.extension().map_or(false, |ext| ext == "json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(workflow) = serde_json::from_str::<Workflow>(&content) {
                            workflows.push(workflow);
                        }
                    }
                }
            }
        }

        Ok(workflows)
    }

    pub async fn get_execution(&self, execution_id: Uuid) -> Result<Option<WorkflowExecution>> {
        let executions_dir = self.config.workflow_dir.join("executions");
        let record_path = executions_dir.join(format!("{}.json", execution_id));

        if record_path.exists() {
            let content = fs::read_to_string(record_path)?;
            let execution = serde_json::from_str::<WorkflowExecution>(&content)?;
            Ok(Some(execution))
        } else {
            Ok(None)
        }
    }
}
