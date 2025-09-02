#[cfg(test)]
mod tests {
    use crate::{config::Config, upload::UploadManager, workflow::WorkflowEngine};

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.upload.max_file_size, 100 * 1024 * 1024);
        assert_eq!(config.workflow.max_concurrent_workflows, 4);
    }

    #[test]
    fn test_upload_manager_creation() {
        let config = Config::default();
        let _upload_manager = UploadManager::new(config.upload);
        // Test that creation doesn't panic
        assert!(true);
    }

    #[test]
    fn test_workflow_engine_creation() {
        let config = Config::default();
        let _workflow_engine = WorkflowEngine::new(config.workflow);
        // Test that creation doesn't panic
        assert!(true);
    }
}
