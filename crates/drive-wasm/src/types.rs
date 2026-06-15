use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmFileNode {
    pub id: String,
    pub name: String,
    pub file_type: String,
    pub parent_id: Option<String>,
    pub size_bytes: u64,
    pub mime_type: Option<String>,
    pub hash_blake3: Option<String>,
    pub encrypted: bool,
    pub encryption_algorithm: Option<String>,
    pub compression_layers: Vec<String>,
    pub tags: Vec<String>,
    pub gps_lat: Option<f64>,
    pub gps_lon: Option<f64>,
    pub created_at: String,
    pub modified_at: String,
    pub is_starred: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SyncBackendType {
    Local,
    GitHub,
    GitLab,
    GoogleDrive,
    GooglePhotos,
    Telegram,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SyncStatus {
    Idle,
    Scanning,
    Compressing,
    Uploading,
    Linking,
    Cleaning,
    Error,
    Done,
    Syncing,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncFileEntry {
    pub id: String,
    pub original_path: String,
    pub size_bytes: u64,
    pub compressed_size_bytes: Option<u64>,
    pub hash_blake3: Option<String>,
    pub backend_type: SyncBackendType,
    pub status: SyncStatus,
    pub synced_at: Option<String>,
    pub error_message: Option<String>,
    pub local_changes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncState {
    pub entries: Vec<SyncFileEntry>,
    pub last_sync_at: Option<String>,
    pub total_bytes: u64,
    pub total_files: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveQuota {
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub file_count: u32,
    pub folder_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncConfigData {
    pub id: String,
    pub name: String,
    pub backend_type: SyncBackendType,
    pub enabled: bool,
    pub base_path: Option<String>,
    pub auto_sync: bool,
    pub compress_before_sync: bool,
    pub max_concurrent_ops: u32,
    pub created_at: String,
    pub updated_at: String,
}

impl WasmFileNode {
    pub fn new(name: String, file_type: String, parent_id: Option<String>) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            file_type,
            parent_id,
            size_bytes: 0,
            mime_type: None,
            hash_blake3: None,
            encrypted: false,
            encryption_algorithm: None,
            compression_layers: vec![],
            tags: vec![],
            gps_lat: None,
            gps_lon: None,
            created_at: now.clone(),
            modified_at: now,
            is_starred: false,
        }
    }
}

impl SyncFileEntry {
    pub fn new(original_path: String, size_bytes: u64, backend_type: SyncBackendType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            original_path,
            size_bytes,
            compressed_size_bytes: None,
            hash_blake3: None,
            backend_type,
            status: SyncStatus::Idle,
            synced_at: None,
            error_message: None,
            local_changes: 0,
        }
    }
}

impl SyncState {
    pub fn new() -> Self {
        Self {
            entries: vec![],
            last_sync_at: None,
            total_bytes: 0,
            total_files: 0,
        }
    }
}
