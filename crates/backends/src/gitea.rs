use crate::util::http_client;
use base64::Engine;
use cybermanju_types::sync::{RemoteFile, StorageBackend, SyncBackendType};
use std::fs;
use std::path::Path;

/// Generic Gitea/Forgejo storage backend.
/// Supports any Gitea or Forgejo instance via configurable base_url.
/// Uses Gitea API v1 (same as Forgejo, used by Codeberg).
///
/// API docs: https://codeberg.org/api/swagger
pub struct GiteaBackend {
    token: String,
    repo: String,
    branch: String,
    base_url: String,
}

impl GiteaBackend {
    pub fn new(token: &str, repo: &str, branch: &str, base_url: Option<&str>) -> Self {
        Self {
            token: token.to_string(),
            repo: repo.to_string(),
            branch: branch.to_string(),
            base_url: base_url
                .unwrap_or("https://try.gitea.io")
                .trim_end_matches('/')
                .to_string(),
        }
    }

    fn contents_url(&self, path: &str) -> String {
        format!(
            "{}/api/v1/repos/{}/contents/{}",
            self.base_url, self.repo, path
        )
    }

    /// Extended contents API — returns richer metadata including LFS info.
    fn contents_ext_url(&self, path: &str) -> String {
        format!(
            "{}/api/v1/repos/{}/contents-ext/{}?includes=file_content,lfs_metadata,commit_metadata",
            self.base_url, self.repo, path
        )
    }

    /// Batch modify multiple files in one API call.
    /// Gitea/Forgejo uniquely supports this — GitHub and GitLab do not.
    fn batch_url(&self) -> String {
        format!("{}/api/v1/repos/{}/contents", self.base_url, self.repo)
    }
}

impl StorageBackend for GiteaBackend {
    fn name(&self) -> &str {
        "Gitea"
    }

    fn backend_type(&self) -> SyncBackendType {
        SyncBackendType::Gitea
    }

    fn upload_file(&self, local_path: &str, remote_path: &str) -> Result<String, String> {
        let data = fs::read(local_path).map_err(|e| format!("read: {}", e))?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
        let url = self.contents_url(remote_path);
        let client = http_client()?;

        // Gitea: POST = create (if file doesn't exist), PUT = update (requires sha)
        // However, PUT without SHA also acts as create (dual behavior).
        // We use: check existence → POST for new, PUT for existing with SHA
        let check = client
            .get(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/json")
            .send();

        let exists = check
            .ok()
            .map(|r| r.status().as_u16() == 200)
            .unwrap_or(false);

        let body = serde_json::json!({
            "message": format!("Upload: {}", remote_path),
            "content": b64,
            "branch": self.branch,
        });

        let resp = if exists {
            let get_resp = client
                .get(&url)
                .header("Authorization", format!("token {}", self.token))
                .header("Accept", "application/json")
                .send()
                .map_err(|e| format!("get existing: {}", e))?;

            let sha = if get_resp.status().is_success() {
                let v: serde_json::Value = get_resp.json().map_err(|e| format!("parse: {}", e))?;
                v["sha"].as_str().unwrap_or("").to_string()
            } else {
                String::new()
            };

            let update_body = serde_json::json!({
                "message": format!("Upload: {}", remote_path),
                "content": b64,
                "branch": self.branch,
                "sha": sha,
            });

            client
                .put(&url)
                .header("Authorization", format!("token {}", self.token))
                .header("Accept", "application/json")
                .json(&update_body)
        } else {
            client
                .post(&url)
                .header("Authorization", format!("token {}", self.token))
                .header("Accept", "application/json")
                .json(&body)
        };

        let r = resp.send().map_err(|e| format!("request: {}", e))?;
        if !r.status().is_success() {
            let status = r.status();
            let body_text = r.text().unwrap_or_default();
            return Err(format!("Gitea upload ({}): {}", status, body_text));
        }

        Ok(format!(
            "{}/{}/raw/branch/{}/{}",
            self.base_url, self.repo, self.branch, remote_path
        ))
    }

    fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String> {
        // Use the /media/ endpoint which auto-detects LFS pointers and resolves them
        let url = format!(
            "{}/api/v1/repos/{}/media/{}?ref={}",
            self.base_url, self.repo, remote_path, self.branch
        );
        let client = http_client()?;
        let resp = client
            .get(&url)
            .header("Authorization", format!("token {}", self.token))
            .send()
            .map_err(|e| format!("request: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Gitea download: HTTP {}", resp.status()));
        }
        let bytes = resp.bytes().map_err(|e| format!("body: {}", e))?;
        if let Some(p) = Path::new(local_path).parent() {
            fs::create_dir_all(p).map_err(|e| format!("mkdir: {}", e))?;
        }
        fs::write(local_path, &bytes).map_err(|e| format!("write: {}", e))?;
        Ok(())
    }

    fn delete_file(&self, remote_path: &str) -> Result<(), String> {
        let url = self.contents_url(remote_path);
        let client = http_client()?;

        let check = client
            .get(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/json")
            .send()
            .map_err(|e| format!("get: {}", e))?;

        if check.status().as_u16() == 404 {
            return Ok(());
        }

        let v: serde_json::Value = check.json().map_err(|e| format!("parse: {}", e))?;
        let sha = v["sha"].as_str().ok_or("no sha")?;

        let body = serde_json::json!({
            "message": format!("Delete: {}", remote_path),
            "sha": sha,
            "branch": self.branch,
        });

        let resp = client
            .delete(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/json")
            .json(&body)
            .send()
            .map_err(|e| format!("delete: {}", e))?;

        if !resp.status().is_success() && resp.status().as_u16() != 204 {
            return Err(format!("Gitea delete: HTTP {}", resp.status()));
        }
        Ok(())
    }

    fn list_files(&self, prefix: &str) -> Result<Vec<RemoteFile>, String> {
        let url = self.contents_url(prefix);
        let client = http_client()?;
        let resp = client
            .get(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/json")
            .send()
            .map_err(|e| format!("request: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Gitea list: HTTP {}", resp.status()));
        }

        let items: Vec<serde_json::Value> = resp.json().map_err(|e| format!("parse: {}", e))?;
        let mut files = vec![];
        for item in &items {
            if item["type"].as_str() == Some("dir") {
                continue;
            }
            files.push(RemoteFile {
                name: item["name"].as_str().unwrap_or("?").into(),
                path: item["path"].as_str().unwrap_or("?").into(),
                size_bytes: item["size"].as_u64().unwrap_or(0),
                modified_at: String::new(),
                url: format!(
                    "{}/api/v1/repos/{}/raw/{}?ref={}",
                    self.base_url,
                    self.repo,
                    item["path"].as_str().unwrap_or("?"),
                    self.branch
                ),
            });
        }
        Ok(files)
    }

    fn get_file_url(&self, remote_path: &str) -> Result<String, String> {
        Ok(format!(
            "{}/{}/raw/branch/{}/{}",
            self.base_url, self.repo, self.branch, remote_path
        ))
    }

    fn test_connection(&self) -> Result<bool, String> {
        let client = http_client()?;
        let resp = client
            .get(&format!("{}/api/v1/user", self.base_url))
            .header("Authorization", format!("token {}", self.token))
            .send()
            .map_err(|e| format!("request: {}", e))?;
        Ok(resp.status().is_success())
    }
}
