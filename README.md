# Rust Automation Orchestrator

A comprehensive automation orchestrator built in Rust with system flow management and upload SOP (Standard Operating Procedure) functionality.

## Features

### 🚀 Core Orchestration
- **Task Management**: Track and manage automation tasks with unique IDs
- **Concurrent Execution**: Control concurrent workflow execution with semaphores
- **Error Handling**: Comprehensive error handling with retry mechanisms
- **Status Tracking**: Real-time task status monitoring and history

### 📁 Upload Management with SOP
- **File Validation**: Size, extension, and integrity validation
- **Automatic Backup**: Configurable backup creation before processing
- **Compression**: Optional file compression with ratio tracking
- **Metadata Extraction**: Automatic metadata generation and tagging
- **Archiving**: Automatic archiving of old files
- **SOP Compliance**: Standardized upload processing procedures

### 🔄 Workflow Engine
- **Dependency Management**: Step dependencies and execution order
- **Conditional Execution**: Conditional step execution based on variables
- **Timeout Control**: Configurable timeouts for each step
- **Retry Logic**: Automatic retry with exponential backoff
- **Variable Substitution**: Dynamic variable replacement in commands

### 🛠️ System Utilities
- **File Operations**: Safe file operations with progress tracking
- **Compression**: Gzip and ZIP compression/decompression
- **Validation**: Comprehensive file and directory validation
- **Logging**: Structured logging with configurable levels

## Installation

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd rust-automation-orchestrator
   ```

2. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Build the project**:
   ```bash
   cargo build --release
   ```

## Configuration

The orchestrator uses a YAML configuration file (`config.yaml`) with the following sections:

### Upload Configuration
```yaml
upload:
  upload_dir: "./uploads"
  max_file_size: 104857600  # 100MB
  allowed_extensions: ["txt", "pdf", "doc", "docx", "zip"]
  compression_enabled: true
  backup_enabled: true
  backup_dir: "./backups"
```

### Workflow Configuration
```yaml
workflow:
  workflow_dir: "./workflows"
  max_concurrent_workflows: 4
  timeout_seconds: 3600  # 1 hour
  retry_attempts: 3
```

### System Configuration
```yaml
system:
  temp_dir: "./temp"
  cache_dir: "./cache"
  max_memory_usage: 1073741824  # 1GB
  cpu_limit: 0.8  # 80%
```

## Usage

### Basic Usage

1. **Process an upload**:
   ```bash
   cargo run -- --upload /path/to/file.txt
   ```

2. **Execute a workflow**:
   ```bash
   cargo run -- --workflow workflows/document_processing.json
   ```

3. **Use custom configuration**:
   ```bash
   cargo run -- --config custom_config.yaml --upload file.txt
   ```

4. **Enable verbose logging**:
   ```bash
   cargo run -- --verbose --upload file.txt
   ```

### Upload SOP Process

The upload process follows a standardized SOP:

1. **Validation**: Check file size, extension, and readability
2. **Backup Creation**: Create backup copy if enabled
3. **File Copy**: Copy to upload directory
4. **Compression**: Compress file if enabled
5. **Metadata Generation**: Generate automatic metadata and tags
6. **Archiving**: Archive old files if needed
7. **Record Creation**: Save upload record for tracking

### Workflow Execution

Workflows are defined in JSON format with the following structure:

```json
{
  "id": "workflow-id",
  "name": "Workflow Name",
  "version": "1.0.0",
  "steps": [
    {
      "id": "step-id",
      "name": "Step Name",
      "step_type": "Command",
      "command": "echo",
      "args": ["Hello World"],
      "timeout": 60,
      "retry_count": 2,
      "depends_on": [],
      "condition": null,
      "output": "step_output"
    }
  ],
  "variables": {
    "input_file": "",
    "output_file": ""
  }
}
```

## API Reference

### AutomationOrchestrator

Main orchestrator class that manages all automation tasks.

```rust
let orchestrator = AutomationOrchestrator::new(config, upload_manager, workflow_engine);

// Process upload
orchestrator.process_upload("/path/to/file.txt").await?;

// Execute workflow
orchestrator.execute_workflow("workflows/my_workflow.json").await?;

// Get task status
let task_info = orchestrator.get_task_status(task_id);

// List active tasks
let tasks = orchestrator.list_active_tasks();
```

### UploadManager

Handles file uploads with SOP compliance.

```rust
let upload_manager = UploadManager::new(upload_config);

// Process upload
let upload_info = upload_manager.process_upload("/path/to/file.txt").await?;

// List uploads
let uploads = upload_manager.list_uploads().await?;

// Get upload details
let upload = upload_manager.get_upload(upload_id).await?;
```

### WorkflowEngine

Executes workflow definitions.

```rust
let workflow_engine = WorkflowEngine::new(workflow_config);

// Execute workflow
let execution = workflow_engine.execute_workflow("workflow.json").await?;

// List workflows
let workflows = workflow_engine.list_workflows().await?;

// Get execution details
let execution = workflow_engine.get_execution(execution_id).await?;
```

## Error Handling

The orchestrator provides comprehensive error handling:

- **File Validation Errors**: Invalid file size, extension, or format
- **Workflow Errors**: Step failures, timeout violations, dependency issues
- **System Errors**: Resource exhaustion, permission issues
- **Network Errors**: Upload/download failures

All errors are logged with appropriate context and can be retried based on configuration.

## Logging

The orchestrator uses structured logging with the following levels:

- **ERROR**: Critical errors that prevent operation
- **WARN**: Warning conditions that don't stop execution
- **INFO**: General information about operations
- **DEBUG**: Detailed debugging information

Logs can be configured to output to console, file, or both.

## Performance Considerations

- **Memory Management**: Configurable memory limits and cleanup
- **Concurrency Control**: Semaphore-based concurrency limiting
- **Resource Monitoring**: CPU and memory usage tracking
- **Efficient File Operations**: Streaming file operations for large files

## Security Features

- **File Validation**: Comprehensive file type and size validation
- **Path Sanitization**: Safe file path handling
- **Permission Checks**: File read/write permission validation
- **Checksum Verification**: File integrity checking

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For support and questions, please open an issue on the GitHub repository.
