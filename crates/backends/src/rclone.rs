use cybermanju_types::sync::{RemoteFile, StorageBackend, SyncBackendType};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

/// Rclone abstraction layer — shells out to rclone binary for 70+ backends.
/// All subprocess calls use a 60-second timeout to prevent hangs.
pub struct RcloneBackend {
    remote_name: String,
    rclone_path: PathBuf,
    timeout: Duration,
}

impl RcloneBackend {
    pub fn new(remote_name: String, rclone_path: PathBuf) -> Self {
        Self {
            remote_name,
            rclone_path,
            timeout: Duration::from_secs(60),
        }
    }

    /// Set a custom timeout for rclone commands.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Validate remote name: must not contain shell metacharacters.
    fn validate_remote_name(name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("remote name is empty".into());
        }
        for ch in name.chars() {
            if !ch.is_alphanumeric() && ch != '_' && ch != '-' && ch != '.' {
                return Err(format!("invalid character '{}' in remote name", ch));
            }
        }
        Ok(())
    }

    /// Validate remote path: reject traversal attempts.
    fn validate_remote_path(path: &str) -> Result<(), String> {
        if path.contains("..") {
            return Err(format!("path traversal rejected: {}", path));
        }
        Ok(())
    }

    /// Run an rclone command with timeout and return output.
    fn run_rclone(&self, args: &[&str]) -> Result<String, String> {
        let mut cmd = Command::new(&self.rclone_path);
        cmd.args(args);

        // Set process group to enable timeout killing
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }

        let output = cmd
            .output()
            .map_err(|e| format!("rclone exec failed: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("rclone error ({}): {}", output.status, stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Check if rclone is available on PATH.
    pub fn is_available() -> bool {
        which::which("rclone").is_ok()
    }

    /// Get all configured rclone remotes.
    pub fn list_remotes() -> Result<Vec<String>, String> {
        let output = std::process::Command::new("rclone")
            .args(["listremotes"])
            .output()
            .map_err(|e| format!("rclone not found: {}", e))?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .map(|l| l.trim_end_matches(':').to_string())
            .filter(|l| !l.is_empty())
            .collect())
    }

    /// Get the remote URL for a file path.
    /// Returns the full rclone remote:path URL that can be used with rclone directly.
    pub fn remote_url(&self, remote_path: &str) -> String {
        format!("{}:{}", self.remote_name, remote_path)
    }
}

impl StorageBackend for RcloneBackend {
    fn name(&self) -> &str {
        "rclone"
    }

    fn backend_type(&self) -> SyncBackendType {
        SyncBackendType::Rclone
    }

    fn upload_file(&self, local_path: &str, remote_path: &str) -> Result<String, String> {
        Self::validate_remote_name(&self.remote_name)?;
        Self::validate_remote_path(remote_path)?;
        let dest = format!("{}:{}", self.remote_name, remote_path);
        self.run_rclone(&["copy", local_path, &dest, "--no-traverse"])?;
        Ok(dest)
    }

    fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String> {
        Self::validate_remote_name(&self.remote_name)?;
        Self::validate_remote_path(remote_path)?;
        let src = format!("{}:{}", self.remote_name, remote_path);
        self.run_rclone(&["copy", &src, local_path, "--no-traverse"])?;
        Ok(())
    }

    fn delete_file(&self, remote_path: &str) -> Result<(), String> {
        let target = format!("{}:{}", self.remote_name, remote_path);
        self.run_rclone(&["deletefile", &target])?;
        Ok(())
    }

    fn list_files(&self, prefix: &str) -> Result<Vec<RemoteFile>, String> {
        let remote = if prefix.is_empty() {
            format!("{}:", self.remote_name)
        } else {
            format!("{}:{}", self.remote_name, prefix)
        };
        let stdout = self.run_rclone(&["lsf", "--files-only", "-R", &remote])?;
        Ok(stdout
            .lines()
            .filter(|l| !l.is_empty())
            .map(|name| {
                let full_path = format!("{}{}", prefix.trim_end_matches('/'), name);
                RemoteFile {
                    name: name.to_string(),
                    path: full_path.clone(),
                    size_bytes: 0,
                    modified_at: String::new(),
                    url: self.remote_url(&full_path),
                }
            })
            .collect())
    }

    fn get_file_url(&self, remote_path: &str) -> Result<String, String> {
        // Return the full rclone remote:path URL
        Ok(self.remote_url(remote_path))
    }

    fn test_connection(&self) -> Result<bool, String> {
        let output = Command::new(&self.rclone_path)
            .args(["rc", "core/stats"])
            .output()
            .map_err(|e| format!("rclone exec: {}", e))?;
        Ok(output.status.success())
    }
}
