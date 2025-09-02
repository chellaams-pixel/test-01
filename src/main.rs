use clap::Parser;
use rust_automation_orchestrator::{
    config::Config,
    orchestrator::AutomationOrchestrator,
    upload::UploadManager,
    workflow::WorkflowEngine,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[clap(short, long, default_value = "config.yaml")]
    config: String,

    /// Workflow file to execute
    #[clap(short, long)]
    workflow: Option<String>,

    /// Upload directory
    #[clap(short, long)]
    upload: Option<String>,

    /// Enable verbose logging
    #[clap(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("rust_automation_orchestrator={}", log_level))
        .init();

    tracing::info!("Starting Rust Automation Orchestrator");

    // Load configuration
    let config = Config::load(&args.config)?;
    tracing::info!("Configuration loaded from {}", args.config);

    // Initialize components
    let upload_manager = UploadManager::new(config.upload.clone());
    let workflow_engine = WorkflowEngine::new(config.workflow.clone());
    let orchestrator = AutomationOrchestrator::new(config, upload_manager, workflow_engine);

    // Execute workflow if specified
    if let Some(workflow_path) = args.workflow {
        tracing::info!("Executing workflow: {}", workflow_path);
        orchestrator.execute_workflow(&workflow_path).await?;
    }

    // Handle upload if specified
    if let Some(upload_path) = args.upload {
        tracing::info!("Processing upload: {}", upload_path);
        orchestrator.process_upload(&upload_path).await?;
    }

    tracing::info!("Automation Orchestrator completed successfully");
    Ok(())
}
