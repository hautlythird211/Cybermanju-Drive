use cybermanju_types::sync::{RemoteFile, StorageBackend, SyncBackendType};
use std::fs;
use std::path::Path;

use crate::util::safe_join;

pub struct LocalBackend {
    base_path: String,
}

impl LocalBackend {
    pub fn new(base_path: &str) -> Self {
        Self { base_path: base_path.to_string() }
    }
}

impl StorageBackend for LocalBackend {
    fn name(&self) -> &str { "Local Storage" }
    fn backend_type(&self) -> SyncBackendType { SyncBackendType::Local }

    fn upload_file(&self, local_path: &str, remote_path: &str) -> Result<String, String> {
        let dest = safe_join(&self.base_path, remote_path)?;
        if let Some(parent) = Path::new(&dest).parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        fs::copy(local_path, &dest).map_err(|e| format!("Failed to copy file: {}", e))?;
        Ok(dest)
    }

    fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String> {
        let src = safe_join(&self.base_path, remote_path)?;
        if Path::new(&src).is_symlink() {
            let link_target = fs::read_link(&src).map_err(|e| format!("Cannot read symlink: {}", e))?;
            let base = Path::new(&self.base_path).canonicalize().map_err(|e| e.to_string())?;
            if !base.join(&link_target).starts_with(&base) {
                return Err("Symlink target outside base path".to_string());
            }
        }
        if let Some(parent) = Path::new(local_path).parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        fs::copy(&src, local_path).map_err(|e| format!("Failed to copy file: {}", e))?;
        Ok(())
    }

    fn delete_file(&self, remote_path: &str) -> Result<(), String> {
        let path = safe_join(&self.base_path, remote_path)?;
        if Path::new(&path).exists() {
            fs::remove_file(&path).map_err(|e| format!("Failed to delete file: {}", e))?;
        }
        Ok(())
    }

    fn list_files(&self, prefix: &str) -> Result<Vec<RemoteFile>, String> {
        let dir = safe_join(&self.base_path, prefix)?;
        if !Path::new(&dir).exists() || !Path::new(&dir).is_dir() {
            return Ok(Vec::new());
        }
        let base = Path::new(&self.base_path).canonicalize().map_err(|e| e.to_string())?;
        let mut files = Vec::new();
        for entry in fs::read_dir(&dir).map_err(|e| format!("Failed to read directory: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            if path.is_file() {
                let meta = entry.metadata().ok();
                let rel = path.strip_prefix(&base).unwrap_or(&path).to_string_lossy().to_string();
                files.push(RemoteFile {
                    name: entry.file_name().to_string_lossy().to_string(),
                    path: rel,
                    size_bytes: meta.as_ref().map(|m| m.len()).unwrap_or(0),
                    modified_at: meta
                        .and_then(|m| m.modified().ok())
                        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
                        .unwrap_or_default(),
                    url: path.to_string_lossy().to_string(),
                });
            }
        }
        Ok(files)
    }

    fn get_file_url(&self, remote_path: &str) -> Result<String, String> {
        safe_join(&self.base_path, remote_path)
    }

    fn test_connection(&self) -> Result<bool, String> {
        let p = Path::new(&self.base_path);
        if !p.exists() {
            fs::create_dir_all(p).map_err(|e| format!("Failed to create base directory: {}", e))?;
        }
        Ok(true)
    }
}
