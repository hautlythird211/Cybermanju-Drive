use crate::util::http_client;
use cybermanju_types::sync::{RemoteFile, StorageBackend, SyncBackendType};
use std::fs;
use std::path::Path;
use urlencoding::encode;

pub struct GoogleDriveBackend {
    token: String,
    folder_id: Option<String>,
}

impl GoogleDriveBackend {
    pub fn new(token: &str, folder_id: Option<&str>) -> Self {
        Self {
            token: token.to_string(),
            folder_id: folder_id.map(String::from),
        }
    }
}

impl StorageBackend for GoogleDriveBackend {
    fn name(&self) -> &str {
        "Google Drive"
    }
    fn backend_type(&self) -> SyncBackendType {
        SyncBackendType::GoogleDrive
    }

    fn upload_file(&self, local_path: &str, _remote_path: &str) -> Result<String, String> {
        let fname = Path::new(local_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let data = fs::read(local_path).map_err(|e| format!("read: {}", e))?;
        let meta = serde_json::json!({ "name": fname, "parents": self.folder_id.as_deref() });
        let form = reqwest::blocking::multipart::Form::new()
            .part(
                "metadata",
                reqwest::blocking::multipart::Part::text(serde_json::to_string(&meta).unwrap())
                    .mime_str("application/json")
                    .unwrap(),
            )
            .part(
                "file",
                reqwest::blocking::multipart::Part::bytes(data)
                    .file_name(fname)
                    .mime_str("application/octet-stream")
                    .unwrap(),
            );
        let client = http_client()?;
        let resp = client
            .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
            .header("Authorization", format!("Bearer {}", self.token))
            .multipart(form)
            .send()
            .map_err(|e| format!("request: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Drive upload: HTTP {}", resp.status()));
        }
        let v: serde_json::Value = resp.json().map_err(|e| format!("parse: {}", e))?;
        let id = v["id"].as_str().ok_or("no id")?;
        Ok(format!("https://drive.google.com/file/d/{}/view", id))
    }

    fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String> {
        let url = format!(
            "https://www.googleapis.com/drive/v3/files/{}?alt=media",
            remote_path
        );
        let client = http_client()?;
        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .map_err(|e| format!("request: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Drive download: HTTP {}", resp.status()));
        }
        let bytes = resp.bytes().map_err(|e| format!("body: {}", e))?;
        if let Some(p) = Path::new(local_path).parent() {
            fs::create_dir_all(p).map_err(|e| format!("mkdir: {}", e))?;
        }
        fs::write(local_path, &bytes).map_err(|e| format!("write: {}", e))?;
        Ok(())
    }

    fn delete_file(&self, remote_path: &str) -> Result<(), String> {
        let client = http_client()?;
        let resp = client
            .delete(&format!(
                "https://www.googleapis.com/drive/v3/files/{}",
                remote_path
            ))
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .map_err(|e| format!("delete: {}", e))?;
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
        let url = format!("https://www.googleapis.com/drive/v3/files?q={}&fields=files(id,name,size,modifiedTime,webContentLink),nextPageToken", encode(&q));
        let client = http_client()?;
        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .map_err(|e| format!("request: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Drive list: HTTP {}", resp.status()));
        }
        let v: serde_json::Value = resp.json().map_err(|e| format!("parse: {}", e))?;
        let arr = v["files"].as_array().cloned().unwrap_or_default();
        Ok(arr
            .into_iter()
            .map(|i| {
                let id = i["id"].as_str().unwrap_or("").to_string();
                RemoteFile {
                    name: i["name"].as_str().unwrap_or("?").into(),
                    path: id.clone(),
                    size_bytes: i["size"]
                        .as_u64()
                        .or_else(|| i["size"].as_str().and_then(|s| s.parse().ok()))
                        .unwrap_or(0),
                    modified_at: i["modifiedTime"].as_str().unwrap_or("").into(),
                    url: format!("https://drive.google.com/file/d/{}/view", id),
                }
            })
            .collect())
    }

    fn get_file_url(&self, remote_path: &str) -> Result<String, String> {
        Ok(format!(
            "https://drive.google.com/file/d/{}/view",
            remote_path
        ))
    }

    fn test_connection(&self) -> Result<bool, String> {
        let client = http_client()?;
        let resp = client
            .get("https://www.googleapis.com/drive/v3/about?fields=user")
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .map_err(|e| format!("request: {}", e))?;
        Ok(resp.status().is_success())
    }
}
