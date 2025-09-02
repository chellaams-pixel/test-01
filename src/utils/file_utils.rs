use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    io::Read,
};
use tracing::info;

pub fn ensure_directory_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
        info!("Created directory: {}", path.display());
    }
    Ok(())
}

pub fn calculate_file_hash(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut hasher = DefaultHasher::new();
    buffer.hash(&mut hasher);
    Ok(format!("{:x}", hasher.finish()))
}

pub fn get_file_size(path: &Path) -> Result<u64> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len())
}

pub fn is_file_readable(path: &Path) -> bool {
    fs::File::open(path).is_ok()
}

pub fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
}

pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

pub fn create_temp_file(prefix: &str, suffix: &str) -> Result<PathBuf> {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("{}_{}{}", prefix, uuid::Uuid::new_v4(), suffix));
    Ok(temp_file)
}

pub fn copy_file_with_progress(src: &Path, dst: &Path) -> Result<u64> {
    ensure_directory_exists(dst.parent().unwrap())?;
    
    let mut src_file = fs::File::open(src)?;
    let mut dst_file = fs::File::create(dst)?;
    
    let bytes_copied = std::io::copy(&mut src_file, &mut dst_file)?;
    info!("Copied {} bytes from {} to {}", bytes_copied, src.display(), dst.display());
    
    Ok(bytes_copied)
}

pub fn remove_file_safely(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)?;
        info!("Removed file: {}", path.display());
    }
    Ok(())
}

pub fn list_files_recursively(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    if dir.exists() && dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                files.push(path);
            } else if path.is_dir() {
                files.extend(list_files_recursively(&path)?);
            }
        }
    }
    
    Ok(files)
}
