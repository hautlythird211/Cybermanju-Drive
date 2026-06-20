use base64::Engine;
use cybermanju_types::sync::{RemoteFile, StorageBackend, SyncBackendType};
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use urlencoding::urlencoding;

fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent("CybermanjuDrive-CLI/0.1")
        .connect_timeout(std::time::Duration::from_secs(15))
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("http client: {}", e))
}

fn safe_join(base: &str, remote: &str) -> Result<String, String> {
    let base = Path::new(base)
        .canonicalize()
        .map_err(|e| format!("bad base path '{}': {}", base, e))?;
    let joined = base.join(remote);
    let canonical = joined
        .canonicalize()
        .map_err(|e| format!("bad joined path '{}': {}", joined.display(), e))?;
    if !canonical.starts_with(&base) {
        return Err("path traversal".into());
    }
    Ok(canonical.to_string_lossy().to_string())
}

// ── Local ─────────────────────────────────────────────────────

pub struct LocalBackend {
    base_path: String,
}

impl LocalBackend {
    pub fn new(base_path: &str) -> Self {
        Self { base_path: base_path.into() }
    }
}

impl StorageBackend for LocalBackend {
    fn name(&self) -> &str { "Local Storage" }
    fn backend_type(&self) -> SyncBackendType { SyncBackendType::Local }

    fn upload_file(&self, local_path: &str, remote_path: &str) -> Result<String, String> {
        let dest = safe_join(&self.base_path, remote_path)?;
        if let Some(p) = Path::new(&dest).parent() {
            fs::create_dir_all(p).map_err(|e| format!("mkdir: {}", e))?;
        }
        fs::copy(local_path, &dest).map_err(|e| format!("copy: {}", e))?;
        Ok(dest)
    }

    fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String> {
        let src = safe_join(&self.base_path, remote_path)?;
        if let Some(p) = Path::new(local_path).parent() {
            fs::create_dir_all(p).map_err(|e| format!("mkdir: {}", e))?;
        }
        fs::copy(&src, local_path).map_err(|e| format!("copy: {}", e))?;
        Ok(())
    }

    fn delete_file(&self, remote_path: &str) -> Result<(), String> {
        let p = safe_join(&self.base_path, remote_path)?;
        let _ = fs::remove_file(&p);
        Ok(())
    }

    fn list_files(&self, prefix: &str) -> Result<Vec<RemoteFile>, String> {
        let dir = safe_join(&self.base_path, prefix)?;
        let d = Path::new(&dir);
        if !d.exists() || !d.is_dir() {
            return Ok(vec![]);
        }
        let mut files = vec![];
        for entry in fs::read_dir(d).map_err(|e| format!("readdir: {}", e))? {
            let e = entry.map_err(|e| format!("entry: {}", e))?;
            let p = e.path();
            if p.is_file() {
                let meta = e.metadata().ok();
                files.push(RemoteFile {
                    name: p.file_name().unwrap_or_default().to_string_lossy().to_string(),
                    path: p.strip_prefix(&self.base_path).unwrap_or(&p).to_string_lossy().to_string(),
                    size_bytes: meta.as_ref().map(|m| m.len()).unwrap_or(0),
                    modified_at: meta.and_then(|m| m.modified().ok())
                        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
                        .unwrap_or_default(),
                    url: p.to_string_lossy().to_string(),
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
            fs::create_dir_all(p).map_err(|e| format!("mkdir: {}", e))?;
        }
        Ok(true)
    }
}

// ── GitHub ────────────────────────────────────────────────────

pub struct GitHubBackend {
    token: String,
    repo: String,
    branch: String,
}

impl GitHubBackend {
    pub fn new(token: &str, repo: &str, branch: &str) -> Self {
        Self { token: token.into(), repo: repo.into(), branch: branch.into() }
    }

    fn parse_repo(&self) -> Result<(String, String), String> {
        let parts: Vec<&str> = self.repo.trim_start_matches('/').splitn(2, '/').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(format!("invalid repo '{}', need owner/repo", self.repo));
        }
        Ok((parts[0].into(), parts[1].into()))
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
        let resp = client.put(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .json(&body)
            .send().map_err(|e| format!("request: {}", e))?;
        let status = resp.status().as_u16();
        let body = resp.text().unwrap_or_default();
        if !(200..300).contains(&status) {
            return Err(format!("GitHub upload ({}): {}", status, body));
        }
        let v: serde_json::Value = serde_json::from_str(&body).map_err(|e| format!("parse: {}", e))?;
        Ok(v["content"]["download_url"].as_str().unwrap_or("").into())
    }

    fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String> {
        let (owner, repo) = self.parse_repo()?;
        let url = format!("https://api.github.com/repos/{}/{}/contents/{}", owner, repo, remote_path);
        let client = http_client()?;
        let resp = client.get(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .send().map_err(|e| format!("request: {}", e))?;
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
        let get = client.get(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .send().map_err(|e| format!("get: {}", e))?;
        if get.status().as_u16() == 404 { return Ok(()); }
        let v: serde_json::Value = get.json().map_err(|e| format!("parse: {}", e))?;
        let sha = v["sha"].as_str().ok_or("no sha")?;
        let body = serde_json::json!({
            "message": format!("CLI delete: {}", remote_path),
            "sha": sha,
            "branch": self.branch,
        });
        let del = client.delete(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .json(&body)
            .send().map_err(|e| format!("delete: {}", e))?;
        if !del.status().is_success() {
            return Err(format!("GitHub delete: HTTP {}", del.status()));
        }
        Ok(())
    }

    fn list_files(&self, prefix: &str) -> Result<Vec<RemoteFile>, String> {
        let (owner, repo) = self.parse_repo()?;
        let url = format!("https://api.github.com/repos/{}/{}/contents/{}", owner, repo, prefix);
        let client = http_client()?;
        let resp = client.get(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .send().map_err(|e| format!("request: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("GitHub list: HTTP {}", resp.status()));
        }
        let items: Vec<serde_json::Value> = resp.json().map_err(|e| format!("parse: {}", e))?;
        let mut files = vec![];
        for item in &items {
            if item["type"].as_str() == Some("dir") { continue; }
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
        let resp = client.get("https://api.github.com/user")
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .send().map_err(|e| format!("request: {}", e))?;
        Ok(resp.status().is_success())
    }
}

// ── GitLab ────────────────────────────────────────────────────

pub struct GitLabBackend {
    token: String,
    project_id: String,
    branch: String,
    base_url: String,
}

impl GitLabBackend {
    pub fn new(token: &str, project_id: &str, branch: &str, base_url: Option<&str>) -> Self {
        Self {
            token: token.into(),
            project_id: project_id.into(),
            branch: branch.into(),
            base_url: base_url.unwrap_or("https://gitlab.com").trim_end_matches('/').into(),
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
        let encoded = urlencoding(remote_path);
        let url = format!("{}/repository/files/{}", self.api_url("repository"), encoded);
        let client = http_client()?;
        let check = client.head(&url)
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
        let encoded = urlencoding(remote_path);
        let url = format!("{}/repository/files/{}/raw?ref={}", self.api_url("repository"), encoded, urlencoding(&self.branch));
        let client = http_client()?;
        let resp = client.get(&url).header("PRIVATE-TOKEN", &self.token).send().map_err(|e| format!("request: {}", e))?;
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
        let encoded = urlencoding(remote_path);
        let url = format!("{}/repository/files/{}", self.api_url("repository"), encoded);
        let body = serde_json::json!({ "branch": self.branch, "commit_message": format!("CLI delete: {}", remote_path) });
        let client = http_client()?;
        let resp = client.delete(&url).header("PRIVATE-TOKEN", &self.token).json(&body).send().map_err(|e| format!("delete: {}", e))?;
        if !resp.status().is_success() && resp.status().as_u16() != 204 {
            return Err(format!("GitLab delete: HTTP {}", resp.status()));
        }
        Ok(())
    }

    fn list_files(&self, prefix: &str) -> Result<Vec<RemoteFile>, String> {
        let q = if prefix.is_empty() { String::new() } else { format!("&path={}", urlencoding(prefix)) };
        let url = format!("{}/repository/tree?ref={}&per_page=100{}", self.api_url("repository"), urlencoding(&self.branch), q);
        let client = http_client()?;
        let resp = client.get(&url).header("PRIVATE-TOKEN", &self.token).send().map_err(|e| format!("request: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("GitLab list: HTTP {}", resp.status()));
        }
        let items: Vec<serde_json::Value> = resp.json().map_err(|e| format!("parse: {}", e))?;
        Ok(items.into_iter().filter(|i| i["type"].as_str() != Some("tree")).map(|i| RemoteFile {
            name: i["name"].as_str().unwrap_or("?").into(),
            path: i["path"].as_str().unwrap_or("?").into(),
            size_bytes: 0,
            modified_at: String::new(),
            url: format!("{}/-/raw/{}/{}", self.base_url, self.branch, i["path"].as_str().unwrap_or("?")),
        }).collect())
    }

    fn get_file_url(&self, remote_path: &str) -> Result<String, String> {
        Ok(format!("{}/-/raw/{}/{}", self.base_url, self.branch, remote_path))
    }

    fn test_connection(&self) -> Result<bool, String> {
        let client = http_client()?;
        let resp = client.get(&format!("{}/api/v4/projects/{}", self.base_url, self.project_id))
            .header("PRIVATE-TOKEN", &self.token)
            .send().map_err(|e| format!("request: {}", e))?;
        Ok(resp.status().is_success())
    }
}

// ── Google Drive ──────────────────────────────────────────────

pub struct GoogleDriveBackend {
    token: String,
    folder_id: Option<String>,
}

impl GoogleDriveBackend {
    pub fn new(token: &str, folder_id: Option<&str>) -> Self {
        Self { token: token.into(), folder_id: folder_id.map(String::from) }
    }
}

impl StorageBackend for GoogleDriveBackend {
    fn name(&self) -> &str { "Google Drive" }
    fn backend_type(&self) -> SyncBackendType { SyncBackendType::GoogleDrive }

    fn upload_file(&self, local_path: &str, _remote_path: &str) -> Result<String, String> {
        let fname = Path::new(local_path).file_name().unwrap_or_default().to_string_lossy().into_owned();
        let data = fs::read(local_path).map_err(|e| format!("read: {}", e))?;
        let meta = serde_json::json!({ "name": fname, "parents": self.folder_id.as_deref() });
        let form = reqwest::blocking::multipart::Form::new()
            .part("metadata", reqwest::blocking::multipart::Part::text(serde_json::to_string(&meta).unwrap()).mime_str("application/json").unwrap())
            .part("file", reqwest::blocking::multipart::Part::bytes(data).file_name(fname).mime_str("application/octet-stream").unwrap());
        let client = http_client()?;
        let resp = client.post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
            .header("Authorization", format!("Bearer {}", self.token))
            .multipart(form)
            .send().map_err(|e| format!("request: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Drive upload: HTTP {}", resp.status()));
        }
        let v: serde_json::Value = resp.json().map_err(|e| format!("parse: {}", e))?;
        let id = v["id"].as_str().ok_or("no id")?;
        Ok(format!("https://drive.google.com/file/d/{}/view", id))
    }

    fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String> {
        let url = format!("https://www.googleapis.com/drive/v3/files/{}?alt=media", remote_path);
        let client = http_client()?;
        let resp = client.get(&url).header("Authorization", format!("Bearer {}", self.token)).send().map_err(|e| format!("request: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Drive download: HTTP {}", resp.status()));
        }
        let bytes = resp.bytes().map_err(|e| format!("body: {}", e))?;
        if let Some(p) = Path::new(local_path).parent() { fs::create_dir_all(p).map_err(|e| format!("mkdir: {}", e))?; }
        fs::write(local_path, &bytes).map_err(|e| format!("write: {}", e))?;
        Ok(())
    }

    fn delete_file(&self, remote_path: &str) -> Result<(), String> {
        let client = http_client()?;
        let resp = client.delete(&format!("https://www.googleapis.com/drive/v3/files/{}", remote_path))
            .header("Authorization", format!("Bearer {}", self.token))
            .send().map_err(|e| format!("delete: {}", e))?;
        if resp.status().as_u16() != 204 && !resp.status().is_success() {
            return Err(format!("Drive delete: HTTP {}", resp.status()));
        }
        Ok(())
    }

    fn list_files(&self, _prefix: &str) -> Result<Vec<RemoteFile>, String> {
        let q = match &self.folder_id {
            Some(id) => format!("'{}' in parents and trashed=false", id),
            None => "trashed=false".into(),
        };
        let url = format!("https://www.googleapis.com/drive/v3/files?q={}&fields=files(id,name,size,modifiedTime,webContentLink),nextPageToken", urlencoding(&q));
        let client = http_client()?;
        let resp = client.get(&url).header("Authorization", format!("Bearer {}", self.token)).send().map_err(|e| format!("request: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Drive list: HTTP {}", resp.status()));
        }
        let v: serde_json::Value = resp.json().map_err(|e| format!("parse: {}", e))?;
        let arr = v["files"].as_array().cloned().unwrap_or_default();
        Ok(arr.into_iter().map(|i| {
            let id = i["id"].as_str().unwrap_or("").to_string();
            RemoteFile {
                name: i["name"].as_str().unwrap_or("?").into(),
                path: id.clone(),
                size_bytes: i["size"].as_u64().or_else(|| i["size"].as_str().and_then(|s| s.parse().ok())).unwrap_or(0),
                modified_at: i["modifiedTime"].as_str().unwrap_or("").into(),
                url: format!("https://drive.google.com/file/d/{}/view", id),
            }
        }).collect())
    }

    fn get_file_url(&self, remote_path: &str) -> Result<String, String> {
        Ok(format!("https://drive.google.com/file/d/{}/view", remote_path))
    }

    fn test_connection(&self) -> Result<bool, String> {
        let client = http_client()?;
        let resp = client.get("https://www.googleapis.com/drive/v3/about?fields=user")
            .header("Authorization", format!("Bearer {}", self.token))
            .send().map_err(|e| format!("request: {}", e))?;
        Ok(resp.status().is_success())
    }
}

// ── Google Photos ─────────────────────────────────────────────

pub struct GooglePhotosBackend {
    token: String,
    album_id: Option<String>,
}

impl GooglePhotosBackend {
    pub fn new(token: &str, album_id: Option<&str>) -> Self {
        Self { token: token.into(), album_id: album_id.map(String::from) }
    }
}

impl StorageBackend for GooglePhotosBackend {
    fn name(&self) -> &str { "Google Photos" }
    fn backend_type(&self) -> SyncBackendType { SyncBackendType::GooglePhotos }

    fn upload_file(&self, local_path: &str, _remote_path: &str) -> Result<String, String> {
        let data = fs::read(local_path).map_err(|e| format!("read: {}", e))?;
        let client = http_client()?;
        let up = client.post("https://photoslibrary.googleapis.com/v1/uploads")
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/octet-stream")
            .header("X-Goog-Upload-Protocol", "raw")
            .body(data)
            .send().map_err(|e| format!("upload token: {}", e))?;
        let token = up.text().map_err(|e| format!("token body: {}", e))?.trim().to_string();
        let mut body = serde_json::json!({ "newMediaItems": [{ "simpleMediaItem": { "uploadToken": token } }] });
        if let Some(ref a) = self.album_id { body["albumId"] = serde_json::json!(a); }
        let cr = client.post("https://photoslibrary.googleapis.com/v1/mediaItems:batchCreate")
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&body)
            .send().map_err(|e| format!("create: {}", e))?;
        if !cr.status().is_success() {
            return Err(format!("Photos create: HTTP {}", cr.status()));
        }
        let v: serde_json::Value = cr.json().map_err(|e| format!("parse: {}", e))?;
        Ok(v["newMediaItemResults"][0]["mediaItem"]["baseUrl"].as_str().unwrap_or("").into())
    }

    fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String> {
        let url = format!("https://photoslibrary.googleapis.com/v1/mediaItems/{}:download", remote_path);
        let client = http_client()?;
        let resp = client.get(&url).header("Authorization", format!("Bearer {}", self.token)).send().map_err(|e| format!("request: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Photos download: HTTP {}", resp.status()));
        }
        let bytes = resp.bytes().map_err(|e| format!("body: {}", e))?;
        if let Some(p) = Path::new(local_path).parent() { fs::create_dir_all(p).map_err(|e| format!("mkdir: {}", e))?; }
        fs::write(local_path, &bytes).map_err(|e| format!("write: {}", e))?;
        Ok(())
    }

    fn delete_file(&self, remote_path: &str) -> Result<(), String> {
        let client = http_client()?;
        let resp = client.delete(&format!("https://photoslibrary.googleapis.com/v1/mediaItems/{}", remote_path))
            .header("Authorization", format!("Bearer {}", self.token))
            .send().map_err(|e| format!("delete: {}", e))?;
        if !resp.status().is_success() { return Err(format!("Photos delete: HTTP {}", resp.status())); }
        Ok(())
    }

    fn list_files(&self, _prefix: &str) -> Result<Vec<RemoteFile>, String> {
        let client = http_client()?;
        let resp = client.get("https://photoslibrary.googleapis.com/v1/mediaItems?pageSize=100")
            .header("Authorization", format!("Bearer {}", self.token))
            .send().map_err(|e| format!("request: {}", e))?;
        if !resp.status().is_success() { return Err(format!("Photos list: HTTP {}", resp.status())); }
        let v: serde_json::Value = resp.json().map_err(|e| format!("parse: {}", e))?;
        let items = v["mediaItems"].as_array().cloned().unwrap_or_default();
        Ok(items.into_iter().map(|i| RemoteFile {
            name: i["filename"].as_str().unwrap_or("?").into(),
            path: i["id"].as_str().unwrap_or("?").into(),
            size_bytes: 0,
            modified_at: i["mediaMetadata"]["creationTime"].as_str().unwrap_or("").into(),
            url: i["baseUrl"].as_str().unwrap_or("").into(),
        }).collect())
    }

    fn get_file_url(&self, remote_path: &str) -> Result<String, String> {
        let client = http_client()?;
        let resp = client.get(&format!("https://photoslibrary.googleapis.com/v1/mediaItems/{}", remote_path))
            .header("Authorization", format!("Bearer {}", self.token))
            .send().map_err(|e| format!("request: {}", e))?;
        if !resp.status().is_success() { return Err(format!("Photos get: HTTP {}", resp.status())); }
        let v: serde_json::Value = resp.json().map_err(|e| format!("parse: {}", e))?;
        Ok(v["baseUrl"].as_str().unwrap_or("").into())
    }

    fn test_connection(&self) -> Result<bool, String> {
        let client = http_client()?;
        let resp = client.get("https://photoslibrary.googleapis.com/v1/albums?pageSize=1")
            .header("Authorization", format!("Bearer {}", self.token))
            .send().map_err(|e| format!("request: {}", e))?;
        Ok(resp.status().is_success())
    }
}

// ── Telegram ──────────────────────────────────────────────────

pub struct TelegramBackend {
    bot_token: String,
    chat_id: String,
}

impl TelegramBackend {
    pub fn new(bot_token: &str, chat_id: &str) -> Self {
        Self { bot_token: bot_token.into(), chat_id: chat_id.into() }
    }
    fn api(&self, method: &str) -> String {
        format!("https://api.telegram.org/bot{}/{}", self.bot_token, method)
    }
}

impl StorageBackend for TelegramBackend {
    fn name(&self) -> &str { "Telegram" }
    fn backend_type(&self) -> SyncBackendType { SyncBackendType::Telegram }

    fn upload_file(&self, local_path: &str, _remote_path: &str) -> Result<String, String> {
        let fname = Path::new(local_path).file_name().unwrap_or_default().to_string_lossy().into_owned();
        let data = fs::read(local_path).map_err(|e| format!("read: {}", e))?;
        let form = reqwest::blocking::multipart::Form::new()
            .text("chat_id", self.chat_id.clone())
            .part("document", reqwest::blocking::multipart::Part::bytes(data).file_name(fname).mime_str("application/octet-stream").unwrap());
        let client = http_client()?;
        let resp = client.post(&self.api("sendDocument")).multipart(form).send().map_err(|e| format!("send: {}", e))?;
        if !resp.status().is_success() { return Err(format!("Telegram upload: HTTP {}", resp.status())); }
        let v: serde_json::Value = resp.json().map_err(|e| format!("parse: {}", e))?;
        Ok(v["result"]["document"]["file_id"].as_str().unwrap_or("").into())
    }

    fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String> {
        let client = http_client()?;
        let resp = client.get(&self.api(&format!("getFile?file_id={}", remote_path))).send().map_err(|e| format!("getFile: {}", e))?;
        if !resp.status().is_success() { return Err(format!("Telegram getFile: HTTP {}", resp.status())); }
        let v: serde_json::Value = resp.json().map_err(|e| format!("parse: {}", e))?;
        let file_path = v["result"]["file_path"].as_str().ok_or("no file_path")?;
        let dl = client.get(&format!("https://api.telegram.org/file/bot{}/{}", self.bot_token, file_path))
            .send().map_err(|e| format!("download: {}", e))?;
        if !dl.status().is_success() { return Err(format!("Telegram download: HTTP {}", dl.status())); }
        let bytes = dl.bytes().map_err(|e| format!("body: {}", e))?;
        if let Some(p) = Path::new(local_path).parent() { fs::create_dir_all(p).map_err(|e| format!("mkdir: {}", e))?; }
        fs::write(local_path, &bytes).map_err(|e| format!("write: {}", e))?;
        Ok(())
    }

    fn delete_file(&self, _remote_path: &str) -> Result<(), String> {
        Err("Telegram cannot delete files".into())
    }

    fn list_files(&self, _prefix: &str) -> Result<Vec<RemoteFile>, String> {
        let client = http_client()?;
        let resp = client.get(&self.api(&format!("getUpdates"))).send().map_err(|e| format!("updates: {}", e))?;
        if !resp.status().is_success() { return Err(format!("Telegram list: HTTP {}", resp.status())); }
        let v: serde_json::Value = resp.json().map_err(|e| format!("parse: {}", e))?;
        let mut files = vec![];
        if let Some(msgs) = v["result"].as_array() {
            for msg in msgs {
                let doc = &msg["message"]["document"];
                if !doc.is_null() {
                    files.push(RemoteFile {
                        name: doc["file_name"].as_str().unwrap_or("?").into(),
                        path: doc["file_id"].as_str().unwrap_or("?").into(),
                        size_bytes: doc["file_size"].as_u64().unwrap_or(0),
                        modified_at: msg["message"]["date"].as_i64().map(|d| chrono::DateTime::from_timestamp(d, 0).map(|t| t.to_rfc3339()).unwrap_or_default()).unwrap_or_default(),
                        url: doc["file_id"].as_str().unwrap_or("").into(),
                    });
                }
            }
        }
        Ok(files)
    }

    fn get_file_url(&self, remote_path: &str) -> Result<String, String> {
        let client = http_client()?;
        let resp = client.get(&self.api(&format!("getFile?file_id={}", remote_path))).send().map_err(|e| format!("getFile: {}", e))?;
        let v: serde_json::Value = resp.json().map_err(|e| format!("parse: {}", e))?;
        let fp = v["result"]["file_path"].as_str().ok_or("no path")?;
        Ok(format!("https://api.telegram.org/file/bot{}/{}", self.bot_token, fp))
    }

    fn test_connection(&self) -> Result<bool, String> {
        let client = http_client()?;
        let resp = client.get(&self.api("getMe")).send().map_err(|e| format!("getMe: {}", e))?;
        Ok(resp.status().is_success())
    }
}

// ── Mega ──────────────────────────────────────────────────────

pub struct MegaBackend {
    email: String,
    password: String,
    client: Mutex<Option<megalib::MegaClient>>,
}

impl MegaBackend {
    pub fn new(email: &str, password: &str) -> Self {
        Self { email: email.into(), password: password.into(), client: Mutex::new(None) }
    }

    fn get_client(&self) -> Result<std::sync::MutexGuard<'_, Option<megalib::MegaClient>>, String> {
        let mut guard = self.client.lock().map_err(|e| format!("lock: {}", e))?;
        if guard.is_none() {
            use std::io::Read;
            let mut c = megalib::MegaClient::new();
            let reader = std::io::BufReader::new(self.email.as_bytes());
            let pw_reader = std::io::BufReader::new(self.password.as_bytes());
            c.login(reader, pw_reader).map_err(|e| format!("mega login: {:?}", e))?;
            *guard = Some(c);
        }
        Ok(guard)
    }
}

impl StorageBackend for MegaBackend {
    fn name(&self) -> &str { "Mega" }
    fn backend_type(&self) -> SyncBackendType { SyncBackendType::Mega }

    fn upload_file(&self, local_path: &str, remote_path: &str) -> Result<String, String> {
        let mut guard = self.get_client()?;
        let c = guard.as_mut().unwrap();
        let data = fs::read(local_path).map_err(|e| format!("read: {}", e))?;
        let fname = Path::new(remote_path).file_name().unwrap_or_default().to_string_lossy().into_owned();
        let mut reader = std::io::BufReader::new(data.as_slice());
        let node = c.upload(reader, Some(&fname), None).map_err(|e| format!("mega upload: {:?}", e))?;
        Ok(format!("mega://{}", node))
    }

    fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String> {
        let mut guard = self.get_client()?;
        let c = guard.as_mut().unwrap();
        let node = c.find_path(remote_path).map_err(|e| format!("mega find: {:?}", e))?;
        let data = c.download(&node).map_err(|e| format!("mega dl: {:?}", e))?;
        if let Some(p) = Path::new(local_path).parent() { fs::create_dir_all(p).map_err(|e| format!("mkdir: {}", e))?; }
        fs::write(local_path, &data).map_err(|e| format!("write: {}", e))?;
        Ok(())
    }

    fn delete_file(&self, remote_path: &str) -> Result<(), String> {
        let mut guard = self.get_client()?;
        let c = guard.as_mut().unwrap();
        let node = c.find_path(remote_path).map_err(|e| format!("mega find: {:?}", e))?;
        c.delete(&node).map_err(|e| format!("mega delete: {:?}", e))?;
        Ok(())
    }

    fn list_files(&self, prefix: &str) -> Result<Vec<RemoteFile>, String> {
        let mut guard = self.get_client()?;
        let c = guard.as_mut().unwrap();
        let path = if prefix.is_empty() { "/" } else { prefix };
        let (children, _) = c.list(path).map_err(|e| format!("mega list: {:?}", e))?;
        Ok(children.into_iter().filter(|n| n.node_type == 0).map(|n| RemoteFile {
            name: n.name.clone(),
            path: n.name.clone(),
            size_bytes: n.size as u64,
            modified_at: String::new(),
            url: format!("mega://{}", n.handle),
        }).collect())
    }

    fn get_file_url(&self, remote_path: &str) -> Result<String, String> {
        let mut guard = self.get_client()?;
        let c = guard.as_mut().unwrap();
        let node = c.find_path(remote_path).map_err(|e| format!("mega find: {:?}", e))?;
        Ok(format!("mega://{}", node))
    }

    fn test_connection(&self) -> Result<bool, String> {
        self.get_client()?;
        Ok(true)
    }
}

// ── Factory ───────────────────────────────────────────────────

pub fn create_backend(backend_type: SyncBackendType, token: &str, config: &serde_json::Value) -> Option<Box<dyn StorageBackend>> {
    match backend_type {
        SyncBackendType::Local => Some(Box::new(LocalBackend::new(token))),
        SyncBackendType::GitHub => {
            let repo = config.get("repo").and_then(|v| v.as_str()).unwrap_or("user/repo");
            let branch = config.get("branch").and_then(|v| v.as_str()).unwrap_or("main");
            Some(Box::new(GitHubBackend::new(token, repo, branch)))
        }
        SyncBackendType::GitLab => {
            let project = config.get("project").and_then(|v| v.as_str()).unwrap_or("0");
            let branch = config.get("branch").and_then(|v| v.as_str()).unwrap_or("main");
            let base = config.get("base_url").and_then(|v| v.as_str());
            Some(Box::new(GitLabBackend::new(token, project, branch, base)))
        }
        SyncBackendType::GoogleDrive => {
            let folder = config.get("folder_id").and_then(|v| v.as_str());
            Some(Box::new(GoogleDriveBackend::new(token, folder)))
        }
        SyncBackendType::GooglePhotos => {
            let album = config.get("album_id").and_then(|v| v.as_str());
            Some(Box::new(GooglePhotosBackend::new(token, album)))
        }
        SyncBackendType::Telegram => {
            let chat = config.get("chat_id").and_then(|v| v.as_str()).unwrap_or("0");
            Some(Box::new(TelegramBackend::new(token, chat)))
        }
        SyncBackendType::Mega => {
            Some(Box::new(MegaBackend::new(token, config.get("password").and_then(|v| v.as_str()).unwrap_or(""))))
        }
    }
}
