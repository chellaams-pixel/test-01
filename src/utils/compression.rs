use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
    io::Write,
};
use tracing::info;

pub fn compress_file_gzip(input_path: &Path, output_path: &Path) -> Result<f64> {
    let input_file = fs::File::open(input_path)?;
    let output_file = fs::File::create(output_path)?;
    
    let mut encoder = flate2::write::GzEncoder::new(output_file, flate2::Compression::default());
    let mut reader = std::io::BufReader::new(input_file);
    
    std::io::copy(&mut reader, &mut encoder)?;
    encoder.finish()?;
    
    let original_size = fs::metadata(input_path)?.len();
    let compressed_size = fs::metadata(output_path)?.len();
    let ratio = original_size as f64 / compressed_size as f64;
    
    info!("Compressed {} to {} (ratio: {:.2})", 
        input_path.display(), output_path.display(), ratio);
    
    Ok(ratio)
}

pub fn decompress_file_gzip(input_path: &Path, output_path: &Path) -> Result<()> {
    let input_file = fs::File::open(input_path)?;
    let output_file = fs::File::create(output_path)?;
    
    let mut decoder = flate2::read::GzDecoder::new(input_file);
    let mut writer = std::io::BufWriter::new(output_file);
    
    std::io::copy(&mut decoder, &mut writer)?;
    
    info!("Decompressed {} to {}", input_path.display(), output_path.display());
    Ok(())
}

pub fn create_zip_archive(files: &[PathBuf], output_path: &Path) -> Result<()> {
    let file = fs::File::create(output_path)?;
    let mut zip = zip::ZipWriter::new(file);
    
    for file_path in files {
        if file_path.exists() {
            let name = file_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            
            zip.start_file(name, zip::write::FileOptions::default())?;
            
            let file_content = fs::read(file_path)?;
            zip.write_all(&file_content)?;
        }
    }
    
    zip.finish()?;
    info!("Created ZIP archive: {}", output_path.display());
    Ok(())
}

pub fn extract_zip_archive(input_path: &Path, output_dir: &Path) -> Result<()> {
    let file = fs::File::open(input_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = output_dir.join(file.name());
        
        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    
    info!("Extracted ZIP archive {} to {}", input_path.display(), output_dir.display());
    Ok(())
}
