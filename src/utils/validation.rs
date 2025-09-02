use anyhow::Result;
use std::path::Path;

pub fn validate_file_size(path: &Path, max_size: u64) -> Result<()> {
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > max_size {
        return Err(anyhow::anyhow!(
            "File size {} exceeds maximum allowed size {}",
            metadata.len(),
            max_size
        ));
    }
    Ok(())
}

pub fn validate_file_extension(path: &Path, allowed_extensions: &[String]) -> Result<()> {
    if let Some(extension) = path.extension() {
        let ext_str = extension.to_string_lossy().to_lowercase();
        if !allowed_extensions.contains(&ext_str.to_string()) {
            return Err(anyhow::anyhow!(
                "File extension '{}' is not allowed",
                ext_str
            ));
        }
    }
    Ok(())
}

pub fn validate_file_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(anyhow::anyhow!("File does not exist: {}", path.display()));
    }
    Ok(())
}

pub fn validate_directory_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(anyhow::anyhow!("Directory does not exist: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(anyhow::anyhow!("Path is not a directory: {}", path.display()));
    }
    Ok(())
}

pub fn validate_file_readable(path: &Path) -> Result<()> {
    std::fs::File::open(path)?;
    Ok(())
}

pub fn validate_file_writable(path: &Path) -> Result<()> {
    if path.exists() {
        // Try to open for writing
        let file = std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(path)?;
        drop(file);
    } else {
        // Try to create the file
        let file = std::fs::File::create(path)?;
        drop(file);
        std::fs::remove_file(path)?;
    }
    Ok(())
}
