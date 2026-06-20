use crate::util::http_client;
use base64::Engine;
use cybermanju_types::sync::{RemoteFile, StorageBackend, SyncBackendType};
use std::fs;
use std::path::Path;

pub struct GitHubBackend {
    token: String,
    repo: String,
    branch: String,
}

impl GitHubBackend {
    pub fn new(token: &str, repo: &str, branch: &str) -> Self {
        Self {
            token: token.to_string(),
            repo: repo.to_string(),
            branch: branch.to_string(),
        }
    }

    fn parse_repo(&self) -> Result<(String, String), String> {
        let parts: Vec<&str> = self.repo.trim_start_matches('/').splitn(2, '/').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(format!("Invalid repo '{}', need owner/repo", self.repo));
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }
}

impl StorageBackend for GitHubBackend {
    fn name(&self) -> &str { "GitHub" }
    fn backend_type(&self) -> SyncBackendType { SyncBackendType::GitHub }

    fn upload_file(&self, local_path: &str, remote_path: &str) -> Result<String, String> {
        let (owner, repo) = self.parse_repo()?;
        let data = fs::read(local_path).map_err(|e| format!("read: {}", e))?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
        let body = serde_json::json!({
            "message": format!("CLI upload: {}", remote_path),
            "content": b64,
            "branch": self.branch,
        });
        let client = http_client()?;
        let url = format!("https://api.github.com/repos/{}/{}/contents/{}", owner, repo, remote_path);
        let resp = client
            .put(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .json(&body)
            .send()
            .map_err(|e| format!("request: {}", e))?;
        let status = resp.status().as_u16();
        let body_text = resp.text().unwrap_or_default();
        if !(200..300).contains(&status) {
            return Err(format!("GitHub upload ({}): {}", status, body_text));
        }
        let v: serde_json::Value = serde_json::from_str(&body_text).map_err(|e| format!("parse: {}", e))?;
        Ok(v["content"]["download_url"].as_str().unwrap_or("").into())
    }

    fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String> {
        let (owner, repo) = self.parse_repo()?;
        let url = format!("https://api.github.com/repos/{}/{}/contents/{}", owner, repo, remote_path);
        let client = http_client()?;
        let resp = client
            .get(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .send()
            .map_err(|e| format!("request: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("GitHub download: HTTP {}", resp.status()));
        }
        let v: serde_json::Value = resp.json().map_err(|e| format!("parse: {}", e))?;
        let content = v["content"].as_str().ok_or("no content")?;
        let clean: String = content.chars().filter(|c| *c != '\n').collect();
        let data = base64::engine::general_purpose::STANDARD.decode(&clean).map_err(|e| format!("b64: {}", e))?;
        if let Some(p) = Path::new(local_path).parent() {
            fs::create_dir_all(p).map_err(|e| format!("mkdir: {}", e))?;
        }
        fs::write(local_path, &data).map_err(|e| format!("write: {}", e))?;
        Ok(())
    }

    fn delete_file(&self, remote_path: &str) -> Result<(), String> {
        let (owner, repo) = self.parse_repo()?;
        let url = format!("https://api.github.com/repos/{}/{}/contents/{}", owner, repo, remote_path);
        let client = http_client()?;
        let get = client
            .get(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .send()
            .map_err(|e| format!("get: {}", e))?;
        if get.status().as_u16() == 404 {
            return Ok(());
        }
        let v: serde_json::Value = get.json().map_err(|e| format!("parse: {}", e))?;
        let sha = v["sha"].as_str().ok_or("no sha")?;
        let body = serde_json::json!({
            "message": format!("CLI delete: {}", remote_path),
            "sha": sha,
            "branch": self.branch,
        });
        let del = client
            .delete(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .json(&body)
            .send()
            .map_err(|e| format!("delete: {}", e))?;
        if !del.status().is_success() {
            return Err(format!("GitHub delete: HTTP {}", del.status()));
        }
        Ok(())
    }

    fn list_files(&self, prefix: &str) -> Result<Vec<RemoteFile>, String> {
        let (owner, repo) = self.parse_repo()?;
        let url = format!("https://api.github.com/repos/{}/{}/contents/{}", owner, repo, prefix);
        let client = http_client()?;
        let resp = client
            .get(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .send()
            .map_err(|e| format!("request: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("GitHub list: HTTP {}", resp.status()));
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
                modified_at: item.get("updated_at").and_then(|v| v.as_str()).unwrap_or("").into(),
                url: item["download_url"].as_str().unwrap_or("").into(),
            });
        }
        Ok(files)
    }

    fn get_file_url(&self, remote_path: &str) -> Result<String, String> {
        let (owner, repo) = self.parse_repo()?;
        Ok(format!("https://raw.githubusercontent.com/{}/{}/{}/{}", owner, repo, self.branch, remote_path))
    }

    fn test_connection(&self) -> Result<bool, String> {
        let client = http_client()?;
        let resp = client
            .get("https://api.github.com/user")
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .send()
            .map_err(|e| format!("request: {}", e))?;
        Ok(resp.status().is_success())
    }
}
