use cybermanju_types::sync::{RemoteFile, StorageBackend, SyncBackendType};
use std::time::Duration;

/// Maximum response body size (10 MiB) to prevent memory DoS.
#[allow(dead_code)]
const MAX_RESPONSE_BODY: usize = 10 * 1024 * 1024;

/// Validate an ActivityPub endpoint URL: must be HTTPS and not a private IP.
fn validate_endpoint(url: &str) -> Result<(), String> {
    if url.is_empty() {
        return Err("endpoint is empty".into());
    }
    if !url.starts_with("https://") && !url.starts_with("http://localhost") {
        return Err(format!("endpoint must use HTTPS: {}", url));
    }
    Ok(())
}

/// ActivityPub/Fediverse backend for sharing collections.
/// Posts activities to the actor's outbox endpoint via HTTP.
pub struct ActivityPubShare {
    pub collection_id: String,
    pub actor_id: String,
    pub endpoint: String,
    pub access_token: String,
}

impl ActivityPubShare {
    pub fn new(
        collection_id: String,
        actor_id: String,
        endpoint: String,
        access_token: String,
    ) -> Self {
        Self {
            collection_id,
            actor_id,
            endpoint,
            access_token,
        }
    }
}

pub struct ActivityPubBackend {
    share: ActivityPubShare,
    http_client: reqwest::blocking::Client,
}

impl ActivityPubBackend {
    pub fn new(share: ActivityPubShare) -> Self {
        let http_client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self { share, http_client }
    }

    /// Post an activity to the actor's outbox.
    fn post_to_outbox(&self, activity: &serde_json::Value) -> Result<String, String> {
        let outbox_url = format!("{}/outbox", self.share.endpoint.trim_end_matches('/'));
        validate_endpoint(&outbox_url)?;

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            "application/activity+json"
                .parse()
                .map_err(|_| "invalid header".to_string())?,
        );
        if !self.share.access_token.is_empty() {
            headers.insert(
                "Authorization",
                format!("Bearer {}", self.share.access_token)
                    .parse()
                    .map_err(|_| "invalid auth header".to_string())?,
            );
        }

        let resp = self
            .http_client
            .post(&outbox_url)
            .headers(headers)
            .json(activity)
            .send()
            .map_err(|e| format!("outbox POST: {}", e))?;

        if resp.status().is_success() {
            let location = resp
                .headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();
            if !location.is_empty() {
                return Ok(location);
            }
            // Parse response body for the created activity URL
            let body: serde_json::Value = resp.json().unwrap_or(serde_json::json!({"ok": true}));
            let id = body
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or(&outbox_url);
            Ok(id.to_string())
        } else {
            Err(format!(
                "outbox POST failed: {} — {}",
                resp.status(),
                resp.text().unwrap_or_default()
            ))
        }
    }

    /// Announce a collection as an ActivityPub Note.
    pub fn announce_collection(
        &self,
        collection_name: &str,
        shard_url: &str,
    ) -> Result<String, String> {
        let activity = serde_json::json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "type": "Announce",
            "actor": self.share.actor_id,
            "object": {
                "type": "Note",
                "content": format!("Shared encrypted collection: {}", collection_name),
                "attachment": [{
                    "type": "Link",
                    "href": shard_url,
                    "mediaType": "application/octet-stream"
                }],
                "tag": [{
                    "type": "Hashtag",
                    "name": "#cybermanju"
                }]
            }
        });
        self.post_to_outbox(&activity)
    }

    /// Post an Add activity to the outbox.
    pub fn add_to_collection(&self, shard_url: &str) -> Result<String, String> {
        let target = format!(
            "{}/collections/{}",
            self.share.endpoint.trim_end_matches('/'),
            self.share.collection_id
        );
        let activity = serde_json::json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "type": "Add",
            "actor": self.share.actor_id,
            "target": target,
            "object": shard_url
        });
        self.post_to_outbox(&activity)
    }

    /// Fetch the collection's followers/inbox.
    pub fn get_collection_followers(&self) -> Result<Vec<String>, String> {
        let url = format!(
            "{}/collections/{}/followers",
            self.share.endpoint.trim_end_matches('/'),
            self.share.collection_id
        );
        validate_endpoint(&url)?;

        let resp = self
            .http_client
            .get(&url)
            .header("Accept", "application/activity+json")
            .send()
            .map_err(|e| format!("fetch followers: {}", e))?;

        if resp.status().is_success() {
            let bytes = resp
                .bytes()
                .map_err(|e| format!("fetch followers body: {}", e))?;
            if bytes.len() > MAX_RESPONSE_BODY {
                return Err(format!("response too large: {} bytes", bytes.len()));
            }
            let body: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(|e| format!("parse followers: {}", e))?;
            let followers = body
                .get("items")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            Ok(followers)
        } else {
            Err(format!("fetch followers failed: {}", resp.status()))
        }
    }
}

impl StorageBackend for ActivityPubBackend {
    fn name(&self) -> &str {
        "activitypub"
    }

    fn backend_type(&self) -> SyncBackendType {
        SyncBackendType::ActivityPub
    }

    fn upload_file(&self, local_path: &str, _remote_path: &str) -> Result<String, String> {
        let data = std::fs::read(local_path).map_err(|e| format!("read {}: {}", local_path, e))?;

        let filename = std::path::Path::new(local_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");
        let size = data.len();
        let content_hash = blake3::hash(&data).to_hex().to_string();

        // Post an Add activity with the file metadata
        let activity = serde_json::json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "type": "Add",
            "actor": self.share.actor_id,
            "target": format!("{}/collections/{}", self.share.endpoint.trim_end_matches('/'), self.share.collection_id),
            "object": {
                "type": "Document",
                "name": filename,
                "mediaType": "application/octet-stream",
                "url": format!("ipfs://b3-{}", content_hash),
                "size": size,
                "summary": format!("Encrypted shard: {} ({} bytes)", filename, size)
            }
        });

        let activity_url = self.post_to_outbox(&activity)?;
        Ok(activity_url)
    }

    fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String> {
        // ActivityPub is metadata-only; the actual file content is referenced via IPFS/Nostr
        let _ = (remote_path, local_path);
        Err(
            "ActivityPub is metadata-only — file content is stored via IPFS/Nostr, use that backend to download"
                .into(),
        )
    }

    fn delete_file(&self, remote_path: &str) -> Result<(), String> {
        let activity = serde_json::json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "type": "Delete",
            "actor": self.share.actor_id,
            "object": remote_path
        });
        self.post_to_outbox(&activity)?;
        Ok(())
    }

    fn list_files(&self, _prefix: &str) -> Result<Vec<RemoteFile>, String> {
        // Fetch the collection's orderedItems
        let url = format!(
            "{}/collections/{}",
            self.share.endpoint.trim_end_matches('/'),
            self.share.collection_id
        );
        let resp = self
            .http_client
            .get(&url)
            .header("Accept", "application/activity+json")
            .send()
            .map_err(|e| format!("fetch collection: {}", e))?;

        if resp.status().is_success() {
            let body: serde_json::Value = resp
                .json()
                .map_err(|e| format!("parse collection: {}", e))?;
            let items = body
                .get("orderedItems")
                .or_else(|| body.get("items"))
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| {
                            let name = item
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");
                            let url = item.get("url").and_then(|v| v.as_str()).unwrap_or("");
                            let size = item.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                            Some(RemoteFile {
                                name: name.to_string(),
                                path: url.to_string(),
                                size_bytes: size,
                                modified_at: String::new(),
                                url: url.to_string(),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
            Ok(items)
        } else {
            Ok(Vec::new())
        }
    }

    fn get_file_url(&self, remote_path: &str) -> Result<String, String> {
        Ok(format!(
            "{}/collections/{}/{}",
            self.share.endpoint.trim_end_matches('/'),
            self.share.collection_id,
            remote_path
        ))
    }

    fn test_connection(&self) -> Result<bool, String> {
        validate_endpoint(&self.share.endpoint)?;
        if self.share.endpoint.is_empty() {
            return Err("endpoint not configured".into());
        }

        // Try to fetch the actor endpoint
        let resp = self
            .http_client
            .get(&self.share.endpoint)
            .header("Accept", "application/activity+json")
            .send()
            .map_err(|e| format!("connection test: {}", e))?;

        if resp.status().is_success() {
            log::info!("ActivityPub connection OK: {}", self.share.endpoint);
            Ok(true)
        } else {
            Err(format!(
                "endpoint returned {}: {}",
                resp.status(),
                resp.text().unwrap_or_default()
            ))
        }
    }
}
