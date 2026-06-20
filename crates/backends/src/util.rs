use std::path::Path;

/// Safely join a base path with a remote path, ensuring the result stays
/// within the base directory. Rejects symlinks that resolve outside base.
pub fn safe_join(base: &str, remote: &str) -> Result<String, String> {
    let base = Path::new(base)
        .canonicalize()
        .map_err(|e| format!("Cannot canonicalize base path '{}': {}", base, e))?;
    let joined = base.join(remote);
    match joined.canonicalize() {
        Ok(canonical) => {
            if !canonical.starts_with(&base) {
                return Err("Path traversal detected".to_string());
            }
            if joined.is_symlink() {
                let link_target = std::fs::read_link(&joined)
                    .map_err(|e| format!("Cannot read symlink: {}", e))?;
                if !base.join(&link_target).starts_with(&base) {
                    return Err("Symlink target outside base path".to_string());
                }
            }
            Ok(canonical.to_string_lossy().to_string())
        }
        Err(_) => {
            if let Some(parent) = joined.parent() {
                if parent.as_os_str().is_empty() {
                    return Ok(base.to_string_lossy().to_string());
                }
                let parent_canonical = parent
                    .canonicalize()
                    .map_err(|e| format!("Cannot canonicalize parent directory: {}", e))?;
                if !parent_canonical.starts_with(&base) {
                    return Err("Path traversal detected in parent directory".to_string());
                }
            }
            let clean = base.join(remote);
            let clean_str = clean.to_string_lossy().to_string();
            let resolved = Path::new(&clean_str);
            for component in resolved.components() {
                if let std::path::Component::ParentDir = component {
                    return Err("Path traversal detected: parent directory component".to_string());
                }
            }
            Ok(clean_str)
        }
    }
}

pub fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent("CybermanjuDrive/0.1")
        .connect_timeout(std::time::Duration::from_secs(15))
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))
}
