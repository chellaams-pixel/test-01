use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub upload: UploadConfig,
    pub workflow: WorkflowConfig,
    pub system: SystemConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadConfig {
    pub upload_dir: PathBuf,
    pub max_file_size: usize,
    pub allowed_extensions: Vec<String>,
    pub compression_enabled: bool,
    pub backup_enabled: bool,
    pub backup_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub workflow_dir: PathBuf,
    pub max_concurrent_workflows: usize,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub temp_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub max_memory_usage: usize,
    pub cpu_limit: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub log_level: String,
    pub log_file: Option<PathBuf>,
    pub enable_console: bool,
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name(path))
            .add_source(config::Environment::with_prefix("AUTOMATION"))
            .build()?;

        Ok(settings.try_deserialize()?)
    }

    pub fn default() -> Self {
        Self {
            upload: UploadConfig::default(),
            workflow: WorkflowConfig::default(),
            system: SystemConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            upload_dir: PathBuf::from("./uploads"),
            max_file_size: 100 * 1024 * 1024, // 100MB
            allowed_extensions: vec![
                "txt".to_string(),
                "pdf".to_string(),
                "doc".to_string(),
                "docx".to_string(),
                "zip".to_string(),
                "tar".to_string(),
                "gz".to_string(),
            ],
            compression_enabled: true,
            backup_enabled: true,
            backup_dir: PathBuf::from("./backups"),
        }
    }
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            workflow_dir: PathBuf::from("./workflows"),
            max_concurrent_workflows: 4,
            timeout_seconds: 3600, // 1 hour
            retry_attempts: 3,
        }
    }
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            temp_dir: PathBuf::from("./temp"),
            cache_dir: PathBuf::from("./cache"),
            max_memory_usage: 1024 * 1024 * 1024, // 1GB
            cpu_limit: 0.8, // 80%
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            log_file: None,
            enable_console: true,
        }
    }
}
