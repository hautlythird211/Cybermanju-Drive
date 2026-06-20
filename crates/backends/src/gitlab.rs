use crate::git_lfs::{GitLfsClient, LfsPlatform};
use crate::util::http_client;
use base64::Engine;
use cybermanju_types::sync::{RemoteFile, StorageBackend, SyncBackendType};
use std::fs;
use std::path::Path;
use urlencoding::encode;

/// LFS threshold for GitLab (files >= 1MB use LFS)
const LFS_SIZE_THRESHOLD: u64 = 1_048_576;

pub struct GitLabBackend {
    token: String,
    project_id: String,
    branch: String,
    base_url: String,
    use_lfs: bool,
}

impl GitLabBackend {
    pub fn new(token: &str, project_id: &str, branch: &str, base_url: Option<&str>) -> Self {
        Self {
            token: token.to_string(),
            project_id: project_id.to_string(),
            branch: branch.to_string(),
            base_url: base_url
                .unwrap_or("https://gitlab.com")
                .trim_end_matches('/')
                .to_string(),
            use_lfs: false,
        }
    }

    pub fn with_lfs(mut self, use_lfs: bool) -> Self {
        self.use_lfs = use_lfs;
        self
    }

    fn api_url(&self, ep: &str) -> String {
        format!(
            "{}/api/v4/projects/{}/{}",
            self.base_url, self.project_id, ep
        )
    }

    fn should_use_lfs(&self, size: u64) -> bool {
        self.use_lfs && size >= LFS_SIZE_THRESHOLD
    }

    /// Internal helper: upload raw content via GitLab Repository Files API.
    fn upload_content(&self, data: &[u8], remote_path: &str) -> Result<String, String> {
        let b64 = base64::engine::general_purpose::STANDARD.encode(data);
        let encoded = encode(remote_path);
        let url = format!(
            "{}/repository/files/{}",
            self.api_url("repository"),
            encoded
        );
        let client = http_client()?;
        let check = client
            .head(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .send();
        let exists = check
            .ok()
            .map(|r| r.status().as_u16() == 200)
            .unwrap_or(false);
        let body = serde_json::json!({
            "branch": self.branch,
            "content": b64,
            "encoding": "base64",
            "commit_message": format!("CLI upload: {}", remote_path),
        });
        let resp = if exists {
            client
                .put(&url)
                .header("PRIVATE-TOKEN", &self.token)
                .json(&body)
        } else {
            client
                .post(&url)
                .header("PRIVATE-TOKEN", &self.token)
                .json(&body)
        };
        let r = resp.send().map_err(|e| format!("request: {}", e))?;
        if !r.status().is_success() {
            return Err(format!("GitLab upload: HTTP {}", r.status()));
        }
        Ok(format!(
            "{}/-/blob/{}/{}",
            self.base_url, self.branch, remote_path
        ))
    }
}

impl StorageBackend for GitLabBackend {
    fn name(&self) -> &str {
        "GitLab"
    }
    fn backend_type(&self) -> SyncBackendType {
        SyncBackendType::GitLab
    }

    fn upload_file(&self, local_path: &str, remote_path: &str) -> Result<String, String> {
        let data = fs::read(local_path).map_err(|e| format!("read: {}", e))?;
        let file_size = data.len() as u64;

        // Use Git LFS for large files
        if self.should_use_lfs(file_size) {
            let lfs = GitLfsClient::new(
                &self.base_url,
                &self.project_id,
                &self.token,
                LfsPlatform::GitLab,
            );
            let oid = lfs.upload_via_lfs(local_path)?;
            let pointer = GitLfsClient::create_pointer(&oid, file_size);
            let pointer_content = pointer.to_string();
            return self.upload_content(pointer_content.as_bytes(), remote_path);
        }

        self.upload_content(&data, remote_path)
    }

    fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String> {
        let encoded = encode(remote_path);
        // Use lfs=true parameter so GitLab resolves LFS pointers to actual content
        let lfs_param = if self.use_lfs { "&lfs=true" } else { "" };
        let url = format!(
            "{}/repository/files/{}/raw?ref={}{}",
            self.api_url("repository"),
            encoded,
            encode(&self.branch),
            lfs_param
        );
        let client = http_client()?;
        let resp = client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .send()
            .map_err(|e| format!("request: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("GitLab download: HTTP {}", resp.status()));
        }
        let bytes = resp.bytes().map_err(|e| format!("body: {}", e))?;

        // Still check for LFS pointer in case lfs=true didn't resolve it
        // (e.g. server doesn't support the parameter)
        if self.use_lfs && GitLfsClient::is_lfs_pointer(&bytes) {
            let pointer_str = String::from_utf8_lossy(&bytes);
            let pointer = cybermanju_types::sync::LfsPointer::from_string(&pointer_str)?;
            let lfs = GitLfsClient::new(
                &self.base_url,
                &self.project_id,
                &self.token,
                LfsPlatform::GitLab,
            );
            return lfs.download_via_lfs(&pointer.oid, pointer.size, local_path);
        }

        if let Some(p) = Path::new(local_path).parent() {
            fs::create_dir_all(p).map_err(|e| format!("mkdir: {}", e))?;
        }
        fs::write(local_path, &bytes).map_err(|e| format!("write: {}", e))?;
        Ok(())
    }

    fn delete_file(&self, remote_path: &str) -> Result<(), String> {
        let encoded = encode(remote_path);
        let url = format!(
            "{}/repository/files/{}",
            self.api_url("repository"),
            encoded
        );
        let body = serde_json::json!({ "branch": self.branch, "commit_message": format!("CLI delete: {}", remote_path) });
        let client = http_client()?;
        let resp = client
            .delete(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .json(&body)
            .send()
            .map_err(|e| format!("delete: {}", e))?;
        if !resp.status().is_success() && resp.status().as_u16() != 204 {
            return Err(format!("GitLab delete: HTTP {}", resp.status()));
        }
        Ok(())
    }

    fn list_files(&self, prefix: &str) -> Result<Vec<RemoteFile>, String> {
        let q = if prefix.is_empty() {
            String::new()
        } else {
            format!("&path={}", encode(prefix))
        };
        let url = format!(
            "{}/repository/tree?ref={}&per_page=100{}",
            self.api_url("repository"),
            encode(&self.branch),
            q
        );
        let client = http_client()?;
        let resp = client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .send()
            .map_err(|e| format!("request: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("GitLab list: HTTP {}", resp.status()));
        }
        let items: Vec<serde_json::Value> = resp.json().map_err(|e| format!("parse: {}", e))?;
        Ok(items
            .into_iter()
            .filter(|i| i["type"].as_str() != Some("tree"))
            .map(|i| RemoteFile {
                name: i["name"].as_str().unwrap_or("?").into(),
                path: i["path"].as_str().unwrap_or("?").into(),
                size_bytes: 0,
                modified_at: String::new(),
                url: format!(
                    "{}/-/raw/{}/{}",
                    self.base_url,
                    self.branch,
                    i["path"].as_str().unwrap_or("?")
                ),
            })
            .collect())
    }

    fn get_file_url(&self, remote_path: &str) -> Result<String, String> {
        Ok(format!(
            "{}/-/raw/{}/{}",
            self.base_url, self.branch, remote_path
        ))
    }

    fn test_connection(&self) -> Result<bool, String> {
        let client = http_client()?;
        let resp = client
            .get(format!(
                "{}/api/v4/projects/{}",
                self.base_url, self.project_id
            ))
            .header("PRIVATE-TOKEN", &self.token)
            .send()
            .map_err(|e| format!("request: {}", e))?;
        Ok(resp.status().is_success())
    }
}
