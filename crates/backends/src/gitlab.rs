use crate::util::http_client;
use base64::Engine;
use cybermanju_types::sync::{RemoteFile, StorageBackend, SyncBackendType};
use std::fs;
use std::path::Path;
use urlencoding::encode;

pub struct GitLabBackend {
    token: String,
    project_id: String,
    branch: String,
    base_url: String,
}

impl GitLabBackend {
    pub fn new(token: &str, project_id: &str, branch: &str, base_url: Option<&str>) -> Self {
        Self {
            token: token.to_string(),
            project_id: project_id.to_string(),
            branch: branch.to_string(),
            base_url: base_url.unwrap_or("https://gitlab.com").trim_end_matches('/').to_string(),
        }
    }

    fn api_url(&self, ep: &str) -> String {
        format!("{}/api/v4/projects/{}/{}", self.base_url, self.project_id, ep)
    }
}

impl StorageBackend for GitLabBackend {
    fn name(&self) -> &str { "GitLab" }
    fn backend_type(&self) -> SyncBackendType { SyncBackendType::GitLab }

    fn upload_file(&self, local_path: &str, remote_path: &str) -> Result<String, String> {
        let data = fs::read(local_path).map_err(|e| format!("read: {}", e))?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
        let encoded = encode(remote_path);
        let url = format!("{}/repository/files/{}", self.api_url("repository"), encoded);
        let client = http_client()?;
        let check = client
            .head(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .send();
        let exists = check.ok().map(|r| r.status().as_u16() == 200).unwrap_or(false);
        let body = serde_json::json!({
            "branch": self.branch,
            "content": b64,
            "encoding": "base64",
            "commit_message": format!("CLI upload: {}", remote_path),
        });
        let resp = if exists {
            client.put(&url).header("PRIVATE-TOKEN", &self.token).json(&body)
        } else {
            client.post(&url).header("PRIVATE-TOKEN", &self.token).json(&body)
        };
        let r = resp.send().map_err(|e| format!("request: {}", e))?;
        if !r.status().is_success() {
            return Err(format!("GitLab upload: HTTP {}", r.status()));
        }
        Ok(format!("{}/-/blob/{}/{}", self.base_url, self.branch, remote_path))
    }

    fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String> {
        let encoded = encode(remote_path);
        let url = format!("{}/repository/files/{}/raw?ref={}", self.api_url("repository"), encoded, encode(&self.branch));
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
        if let Some(p) = Path::new(local_path).parent() {
            fs::create_dir_all(p).map_err(|e| format!("mkdir: {}", e))?;
        }
        fs::write(local_path, &bytes).map_err(|e| format!("write: {}", e))?;
        Ok(())
    }

    fn delete_file(&self, remote_path: &str) -> Result<(), String> {
        let encoded = encode(remote_path);
        let url = format!("{}/repository/files/{}", self.api_url("repository"), encoded);
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
        let q = if prefix.is_empty() { String::new() } else { format!("&path={}", encode(prefix)) };
        let url = format!("{}/repository/tree?ref={}&per_page=100{}", self.api_url("repository"), encode(&self.branch), q);
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
                url: format!("{}/-/raw/{}/{}", self.base_url, self.branch, i["path"].as_str().unwrap_or("?")),
            })
            .collect())
    }

    fn get_file_url(&self, remote_path: &str) -> Result<String, String> {
        Ok(format!("{}/-/raw/{}/{}", self.base_url, self.branch, remote_path))
    }

    fn test_connection(&self) -> Result<bool, String> {
        let client = http_client()?;
        let resp = client
            .get(&format!("{}/api/v4/projects/{}", self.base_url, self.project_id))
            .header("PRIVATE-TOKEN", &self.token)
            .send()
            .map_err(|e| format!("request: {}", e))?;
        Ok(resp.status().is_success())
    }
}
