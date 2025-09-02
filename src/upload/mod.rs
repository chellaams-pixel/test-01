use crate::config::UploadConfig;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tracing::info;
use uuid::Uuid;
use walkdir::WalkDir;

#[derive(Debug)]
pub struct UploadManager {
    config: UploadConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadInfo {
    pub id: Uuid,
    pub filename: String,
    pub original_path: PathBuf,
    pub processed_path: PathBuf,
    pub file_size: u64,
    pub mime_type: String,
    pub upload_timestamp: DateTime<Utc>,
    pub processing_status: ProcessingStatus,
    pub metadata: UploadMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadMetadata {
    pub checksum: String,
    pub compression_ratio: Option<f64>,
    pub backup_path: Option<PathBuf>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Archived,
}

impl UploadManager {
    pub fn new(config: UploadConfig) -> Self {
        Self { config }
    }

    pub async fn process_upload(&self, upload_path: &str) -> Result<UploadInfo> {
        let path = Path::new(upload_path);
        
        if !path.exists() {
            return Err(anyhow::anyhow!("Upload path does not exist: {}", upload_path));
        }

        let upload_id = Uuid::new_v4();
        info!("Processing upload {}: {}", upload_id, upload_path);

        // Step 1: Validate upload
        self.validate_upload(path).await?;

        // Step 2: Create upload info
        let mut upload_info = self.create_upload_info(upload_id, path).await?;

        // Step 3: Execute SOP (Standard Operating Procedure)
        self.execute_upload_sop(&mut upload_info).await?;

        // Step 4: Save upload record
        self.save_upload_record(&upload_info).await?;

        info!("Upload {} processed successfully", upload_id);
        Ok(upload_info)
    }

    async fn validate_upload(&self, path: &Path) -> Result<()> {
        info!("Validating upload: {}", path.display());

        // Check file size
        let metadata = fs::metadata(path)?;
        if metadata.len() > self.config.max_file_size as u64 {
            return Err(anyhow::anyhow!(
                "File size {} exceeds maximum allowed size {}",
                metadata.len(),
                self.config.max_file_size
            ));
        }

        // Check file extension
        if let Some(extension) = path.extension() {
            let ext_str = extension.to_string_lossy().to_lowercase();
            if !self.config.allowed_extensions.contains(&ext_str.to_string()) {
                return Err(anyhow::anyhow!(
                    "File extension '{}' is not allowed",
                    ext_str
                ));
            }
        }

        // Check if file is readable
        fs::File::open(path)?;

        info!("Upload validation passed");
        Ok(())
    }

    async fn create_upload_info(&self, upload_id: Uuid, path: &Path) -> Result<UploadInfo> {
        let metadata = fs::metadata(path)?;
        let filename = path.file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?
            .to_string();

        let mime_type = self.detect_mime_type(path)?;
        let checksum = self.calculate_checksum(path).await?;

        Ok(UploadInfo {
            id: upload_id,
            filename: filename.clone(),
            original_path: path.to_path_buf(),
            processed_path: self.config.upload_dir.join(&filename),
            file_size: metadata.len(),
            mime_type,
            upload_timestamp: Utc::now(),
            processing_status: ProcessingStatus::Pending,
            metadata: UploadMetadata {
                checksum,
                compression_ratio: None,
                backup_path: None,
                tags: Vec::new(),
                notes: None,
            },
        })
    }

    async fn execute_upload_sop(&self, upload_info: &mut UploadInfo) -> Result<()> {
        info!("Executing upload SOP for {}", upload_info.id);

        upload_info.processing_status = ProcessingStatus::Processing;

        // SOP Step 1: Create backup if enabled
        if self.config.backup_enabled {
            self.create_backup(upload_info).await?;
        }

        // SOP Step 2: Copy file to upload directory
        self.copy_to_upload_dir(upload_info).await?;

        // SOP Step 3: Compress if enabled
        if self.config.compression_enabled {
            self.compress_file(upload_info).await?;
        }

        // SOP Step 4: Generate metadata
        self.generate_metadata(upload_info).await?;

        // SOP Step 5: Archive if needed
        self.archive_if_needed(upload_info).await?;

        upload_info.processing_status = ProcessingStatus::Completed;
        info!("Upload SOP completed for {}", upload_info.id);

        Ok(())
    }

    async fn create_backup(&self, upload_info: &mut UploadInfo) -> Result<()> {
        let backup_filename = format!("{}_{}.bak", 
            upload_info.id, 
            upload_info.upload_timestamp.format("%Y%m%d_%H%M%S")
        );
        let backup_path = self.config.backup_dir.join(backup_filename);

        // Ensure backup directory exists
        fs::create_dir_all(&self.config.backup_dir)?;

        // Copy file to backup location
        fs::copy(&upload_info.original_path, &backup_path)?;
        upload_info.metadata.backup_path = Some(backup_path);

        info!("Backup created: {}", upload_info.metadata.backup_path.as_ref().unwrap().display());
        Ok(())
    }

    async fn copy_to_upload_dir(&self, upload_info: &mut UploadInfo) -> Result<()> {
        // Ensure upload directory exists
        fs::create_dir_all(&self.config.upload_dir)?;

        // Copy file to upload directory
        fs::copy(&upload_info.original_path, &upload_info.processed_path)?;

        info!("File copied to upload directory: {}", upload_info.processed_path.display());
        Ok(())
    }

    async fn compress_file(&self, upload_info: &mut UploadInfo) -> Result<()> {
        let original_size = upload_info.file_size;
        
        // Create compressed file path
        let compressed_path = upload_info.processed_path.with_extension("gz");
        
        // Compress file using gzip
        let input = fs::File::open(&upload_info.processed_path)?;
        let output = fs::File::create(&compressed_path)?;
        
        let mut encoder = flate2::write::GzEncoder::new(output, flate2::Compression::default());
        std::io::copy(&mut std::io::BufReader::new(input), &mut encoder)?;
        encoder.finish()?;

        // Update processed path and calculate compression ratio
        let compressed_size = fs::metadata(&compressed_path)?.len();
        let compression_ratio = original_size as f64 / compressed_size as f64;
        
        upload_info.processed_path = compressed_path;
        upload_info.metadata.compression_ratio = Some(compression_ratio);

        info!("File compressed with ratio: {:.2}", compression_ratio);
        Ok(())
    }

    async fn generate_metadata(&self, upload_info: &mut UploadInfo) -> Result<()> {
        // Add automatic tags based on file type
        if let Some(extension) = upload_info.processed_path.extension() {
            let ext_str = extension.to_string_lossy().to_lowercase();
            upload_info.metadata.tags.push(format!("ext:{}", ext_str));
        }

        // Add size-based tags
        if upload_info.file_size > 10 * 1024 * 1024 { // 10MB
            upload_info.metadata.tags.push("large_file".to_string());
        }

        // Add timestamp tag
        upload_info.metadata.tags.push(format!("uploaded:{}", 
            upload_info.upload_timestamp.format("%Y-%m-%d")));

        info!("Metadata generated with {} tags", upload_info.metadata.tags.len());
        Ok(())
    }

    async fn archive_if_needed(&self, upload_info: &mut UploadInfo) -> Result<()> {
        // Archive files older than 30 days
        let thirty_days_ago = Utc::now() - chrono::Duration::days(30);
        
        if upload_info.upload_timestamp < thirty_days_ago {
            let archive_dir = self.config.upload_dir.join("archive");
            fs::create_dir_all(&archive_dir)?;
            
            let archive_path = archive_dir.join(&upload_info.filename);
            fs::rename(&upload_info.processed_path, &archive_path)?;
            upload_info.processed_path = archive_path;
            upload_info.processing_status = ProcessingStatus::Archived;
            
            info!("File archived: {}", upload_info.processed_path.display());
        }

        Ok(())
    }

    async fn save_upload_record(&self, upload_info: &UploadInfo) -> Result<()> {
        let records_dir = self.config.upload_dir.join("records");
        fs::create_dir_all(&records_dir)?;
        
        let record_path = records_dir.join(format!("{}.json", upload_info.id));
        let record_json = serde_json::to_string_pretty(upload_info)?;
        fs::write(record_path, record_json)?;

        info!("Upload record saved for {}", upload_info.id);
        Ok(())
    }

    fn detect_mime_type(&self, path: &Path) -> Result<String> {
        if let Some(extension) = path.extension() {
            let ext_str = extension.to_string_lossy().to_lowercase();
            match ext_str.as_str() {
                "txt" => Ok("text/plain".to_string()),
                "pdf" => Ok("application/pdf".to_string()),
                "doc" | "docx" => Ok("application/msword".to_string()),
                "zip" => Ok("application/zip".to_string()),
                "tar" => Ok("application/x-tar".to_string()),
                "gz" => Ok("application/gzip".to_string()),
                "json" => Ok("application/json".to_string()),
                "yaml" | "yml" => Ok("application/x-yaml".to_string()),
                _ => Ok("application/octet-stream".to_string()),
            }
        } else {
            Ok("application/octet-stream".to_string())
        }
    }

    async fn calculate_checksum(&self, path: &Path) -> Result<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::io::Read;

        let mut file = fs::File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let mut hasher = DefaultHasher::new();
        buffer.hash(&mut hasher);
        Ok(format!("{:x}", hasher.finish()))
    }

    pub async fn list_uploads(&self) -> Result<Vec<UploadInfo>> {
        let mut uploads = Vec::new();
        let records_dir = self.config.upload_dir.join("records");

        if records_dir.exists() {
            for entry in WalkDir::new(&records_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
            {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if let Ok(upload_info) = serde_json::from_str::<UploadInfo>(&content) {
                        uploads.push(upload_info);
                    }
                }
            }
        }

        Ok(uploads)
    }

    pub async fn get_upload(&self, upload_id: Uuid) -> Result<Option<UploadInfo>> {
        let records_dir = self.config.upload_dir.join("records");
        let record_path = records_dir.join(format!("{}.json", upload_id));

        if record_path.exists() {
            let content = fs::read_to_string(record_path)?;
            let upload_info = serde_json::from_str::<UploadInfo>(&content)?;
            Ok(Some(upload_info))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_upload(&self, upload_id: Uuid) -> Result<()> {
        if let Some(upload_info) = self.get_upload(upload_id).await? {
            // Remove processed file
            if upload_info.processed_path.exists() {
                fs::remove_file(&upload_info.processed_path)?;
            }

            // Remove backup if exists
            if let Some(backup_path) = upload_info.metadata.backup_path {
                if backup_path.exists() {
                    fs::remove_file(backup_path)?;
                }
            }

            // Remove record
            let records_dir = self.config.upload_dir.join("records");
            let record_path = records_dir.join(format!("{}.json", upload_id));
            if record_path.exists() {
                fs::remove_file(record_path)?;
            }

            info!("Upload {} deleted successfully", upload_id);
        }

        Ok(())
    }
}
