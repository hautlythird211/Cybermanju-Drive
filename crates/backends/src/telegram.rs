use crate::util::http_client;
use cybermanju_types::sync::{RemoteFile, StorageBackend, SyncBackendType};
use std::fs;
use std::path::Path;

pub struct TelegramBackend {
    bot_token: String,
    chat_id: String,
}

impl TelegramBackend {
    pub fn new(bot_token: &str, chat_id: &str) -> Self {
        Self {
            bot_token: bot_token.to_string(),
            chat_id: chat_id.to_string(),
        }
    }

    fn api(&self, method: &str) -> String {
        format!("https://api.telegram.org/bot{}/{}", self.bot_token, method)
    }
}

impl StorageBackend for TelegramBackend {
    fn name(&self) -> &str {
        "Telegram"
    }
    fn backend_type(&self) -> SyncBackendType {
        SyncBackendType::Telegram
    }

    fn upload_file(&self, local_path: &str, _remote_path: &str) -> Result<String, String> {
        let fname = Path::new(local_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let data = fs::read(local_path).map_err(|e| format!("read: {}", e))?;
        let form = reqwest::blocking::multipart::Form::new()
            .text("chat_id", self.chat_id.clone())
            .part(
                "document",
                reqwest::blocking::multipart::Part::bytes(data)
                    .file_name(fname)
                    .mime_str("application/octet-stream")
                    .unwrap(),
            );
        let client = http_client()?;
        let resp = client
            .post(&self.api("sendDocument"))
            .multipart(form)
            .send()
            .map_err(|e| format!("send: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Telegram upload: HTTP {}", resp.status()));
        }
        let v: serde_json::Value = resp.json().map_err(|e| format!("parse: {}", e))?;
        let file_id = v["result"]["document"]["file_id"]
            .as_str()
            .unwrap_or("")
            .to_string();
        if file_id.is_empty() {
            // Try photo/audio/video fallback
            let fallback = v["result"]["photo"]
                .as_array()
                .and_then(|arr| arr.last())
                .and_then(|p| p["file_id"].as_str())
                .or_else(|| v["result"]["audio"]["file_id"].as_str())
                .or_else(|| v["result"]["video"]["file_id"].as_str())
                .unwrap_or("");
            if fallback.is_empty() {
                return Err(
                    "No file_id in Telegram response (media type may not be supported)".to_string(),
                );
            }
            return Ok(fallback.to_string());
        }
        Ok(file_id)
    }

    fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String> {
        let client = http_client()?;
        let resp = client
            .get(&self.api(&format!("getFile?file_id={}", remote_path)))
            .send()
            .map_err(|e| format!("getFile: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Telegram getFile: HTTP {}", resp.status()));
        }
        let v: serde_json::Value = resp.json().map_err(|e| format!("parse: {}", e))?;
        let file_path = v["result"]["file_path"].as_str().ok_or("no file_path")?;
        let dl = client
            .get(&format!(
                "https://api.telegram.org/file/bot{}/{}",
                self.bot_token, file_path
            ))
            .send()
            .map_err(|e| format!("download: {}", e))?;
        if !dl.status().is_success() {
            return Err(format!("Telegram download: HTTP {}", dl.status()));
        }
        let bytes = dl.bytes().map_err(|e| format!("body: {}", e))?;
        if let Some(p) = Path::new(local_path).parent() {
            fs::create_dir_all(p).map_err(|e| format!("mkdir: {}", e))?;
        }
        fs::write(local_path, &bytes).map_err(|e| format!("write: {}", e))?;
        Ok(())
    }

    fn delete_file(&self, _remote_path: &str) -> Result<(), String> {
        Err("Telegram cannot delete files".into())
    }

    fn list_files(&self, _prefix: &str) -> Result<Vec<RemoteFile>, String> {
        let client = http_client()?;
        let resp = client
            .get(&self.api("getUpdates"))
            .send()
            .map_err(|e| format!("updates: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Telegram list: HTTP {}", resp.status()));
        }
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
                        modified_at: msg["message"]["date"]
                            .as_i64()
                            .and_then(|d| {
                                chrono::DateTime::from_timestamp(d, 0).map(|t| t.to_rfc3339())
                            })
                            .unwrap_or_default(),
                        url: doc["file_id"].as_str().unwrap_or("").into(),
                    });
                }
            }
        }
        Ok(files)
    }

    fn get_file_url(&self, remote_path: &str) -> Result<String, String> {
        let client = http_client()?;
        let resp = client
            .get(&self.api(&format!("getFile?file_id={}", remote_path)))
            .send()
            .map_err(|e| format!("getFile: {}", e))?;
        let v: serde_json::Value = resp.json().map_err(|e| format!("parse: {}", e))?;
        let fp = v["result"]["file_path"].as_str().ok_or("no path")?;
        Ok(format!(
            "https://api.telegram.org/file/bot{}/{}",
            self.bot_token, fp
        ))
    }

    fn test_connection(&self) -> Result<bool, String> {
        let client = http_client()?;
        let resp = client
            .get(&self.api("getMe"))
            .send()
            .map_err(|e| format!("getMe: {}", e))?;
        let status = resp.status().as_u16();
        let body = resp.text().map_err(|e| format!("read: {}", e))?;
        if status != 200 {
            return Err(format!(
                "Telegram connection test failed (HTTP {}): {}",
                status, body
            ));
        }
        let resp_json: serde_json::Value =
            serde_json::from_str(&body).map_err(|e| format!("parse: {}", e))?;
        if !resp_json["ok"].as_bool().unwrap_or(false) {
            let desc = resp_json["description"].as_str().unwrap_or("Unknown error");
            return Err(format!("Telegram API error: {}", desc));
        }
        let bot_name = resp_json["result"]["username"]
            .as_str()
            .unwrap_or("unknown");
        log::info!("Telegram bot connected: @{}", bot_name);
        Ok(true)
    }
}
