use crate::util::http_client;
use cybermanju_types::sync::{RemoteFile, StorageBackend, SyncBackendType};
use std::fs;
use std::path::Path;

pub struct GooglePhotosBackend {
    token: String,
    album_id: Option<String>,
}

impl GooglePhotosBackend {
    pub fn new(token: &str, album_id: Option<&str>) -> Self {
        Self {
            token: token.to_string(),
            album_id: album_id.map(String::from),
        }
    }
}

impl StorageBackend for GooglePhotosBackend {
    fn name(&self) -> &str { "Google Photos" }
    fn backend_type(&self) -> SyncBackendType { SyncBackendType::GooglePhotos }

    fn upload_file(&self, local_path: &str, _remote_path: &str) -> Result<String, String> {
        let data = fs::read(local_path).map_err(|e| format!("read: {}", e))?;
        let client = http_client()?;
        let up = client
            .post("https://photoslibrary.googleapis.com/v1/uploads")
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/octet-stream")
            .header("X-Goog-Upload-Protocol", "raw")
            .body(data)
            .send()
            .map_err(|e| format!("upload token: {}", e))?;
        let token = up.text().map_err(|e| format!("token body: {}", e))?.trim().to_string();
        let mut body = serde_json::json!({ "newMediaItems": [{ "simpleMediaItem": { "uploadToken": token } }] });
        if let Some(ref a) = self.album_id {
            body["albumId"] = serde_json::json!(a);
        }
        let cr = client
            .post("https://photoslibrary.googleapis.com/v1/mediaItems:batchCreate")
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&body)
            .send()
            .map_err(|e| format!("create: {}", e))?;
        if !cr.status().is_success() {
            return Err(format!("Photos create: HTTP {}", cr.status()));
        }
        let v: serde_json::Value = cr.json().map_err(|e| format!("parse: {}", e))?;
        Ok(v["newMediaItemResults"][0]["mediaItem"]["baseUrl"].as_str().unwrap_or("").into())
    }

    fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String> {
        let client = http_client()?;
        let meta_url = format!("https://photoslibrary.googleapis.com/v1/mediaItems/{}", remote_path);
        let meta_resp = client
            .get(&meta_url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .map_err(|e| format!("request: {}", e))?;
        if !meta_resp.status().is_success() {
            return Err(format!("Photos get: HTTP {}", meta_resp.status()));
        }
        let v: serde_json::Value = meta_resp.json().map_err(|e| format!("parse: {}", e))?;
        let base_url = v["baseUrl"].as_str().ok_or("no baseUrl in response")?;
        let dl_url = format!("{}=d", base_url);
        let dl_resp = client
            .get(&dl_url)
            .send()
            .map_err(|e| format!("download: {}", e))?;
        if !dl_resp.status().is_success() {
            return Err(format!("Photos download: HTTP {}", dl_resp.status()));
        }
        let bytes = dl_resp.bytes().map_err(|e| format!("body: {}", e))?;
        if let Some(p) = Path::new(local_path).parent() {
            fs::create_dir_all(p).map_err(|e| format!("mkdir: {}", e))?;
        }
        fs::write(local_path, &bytes).map_err(|e| format!("write: {}", e))?;
        Ok(())
    }

    fn delete_file(&self, remote_path: &str) -> Result<(), String> {
        let client = http_client()?;
        let resp = client
            .delete(&format!("https://photoslibrary.googleapis.com/v1/mediaItems/{}", remote_path))
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .map_err(|e| format!("delete: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Photos delete: HTTP {}", resp.status()));
        }
        Ok(())
    }

    fn list_files(&self, _prefix: &str) -> Result<Vec<RemoteFile>, String> {
        let client = http_client()?;
        let resp = client
            .get("https://photoslibrary.googleapis.com/v1/mediaItems?pageSize=100")
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .map_err(|e| format!("request: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Photos list: HTTP {}", resp.status()));
        }
        let v: serde_json::Value = resp.json().map_err(|e| format!("parse: {}", e))?;
        let items = v["mediaItems"].as_array().cloned().unwrap_or_default();
        Ok(items
            .into_iter()
            .map(|i| RemoteFile {
                name: i["filename"].as_str().unwrap_or("?").into(),
                path: i["id"].as_str().unwrap_or("?").into(),
                size_bytes: 0,
                modified_at: i["mediaMetadata"]["creationTime"].as_str().unwrap_or("").into(),
                url: i["baseUrl"].as_str().unwrap_or("").into(),
            })
            .collect())
    }

    fn get_file_url(&self, remote_path: &str) -> Result<String, String> {
        let client = http_client()?;
        let resp = client
            .get(&format!("https://photoslibrary.googleapis.com/v1/mediaItems/{}", remote_path))
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .map_err(|e| format!("request: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Photos get: HTTP {}", resp.status()));
        }
        let v: serde_json::Value = resp.json().map_err(|e| format!("parse: {}", e))?;
        Ok(v["baseUrl"].as_str().unwrap_or("").into())
    }

    fn test_connection(&self) -> Result<bool, String> {
        let client = http_client()?;
        let resp = client
            .get("https://photoslibrary.googleapis.com/v1/albums?pageSize=1")
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .map_err(|e| format!("request: {}", e))?;
        Ok(resp.status().is_success())
    }
}
