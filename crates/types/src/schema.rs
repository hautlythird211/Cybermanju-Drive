use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FileNode {
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
    pub thumbnail_path: Option<String>,
    pub context_data: Option<serde_json::Value>,
    pub tags: Vec<String>,
    pub collection_ids: Vec<String>,
    pub face_group_ids: Vec<String>,
    pub loose_group_ids: Vec<String>,
    pub gps_lat: Option<f64>,
    pub gps_lon: Option<f64>,
    pub created_at: String,
    pub modified_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub name: String,
    pub account_type: String,
    pub path: Option<String>,
    pub color: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub collection_type: String,
    pub color: String,
    pub description: Option<String>,
    pub item_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CollectionItem {
    pub id: String,
    pub collection_id: String,
    pub file_id: String,
    pub note: Option<String>,
    pub added_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FaceGroup {
    pub id: String,
    pub name: String,
    pub file_ids: Vec<String>,
    pub centroid_embedding: Option<Vec<f32>>,
    pub binary_hash: Option<u64>,
    pub cohesion: Option<f32>,
    pub embedding_count: u32,
    pub algorithm: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionKey {
    pub id: String,
    pub algorithm: String,
    pub public_key: String,
    pub private_key: String,
    pub label: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LooseGroup {
    pub id: String,
    pub name: String,
    pub color: String,
    pub file_ids: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub role: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserFilePermission {
    pub id: String,
    pub user_id: String,
    pub file_id: String,
    pub access: String,
    pub granted_by: String,
    pub granted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub id: String,
    pub file_id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub place_name: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TrashItem {
    pub id: String,
    pub original_file: FileNode,
    pub deleted_at: String,
    pub deleted_by: Option<String>,
    pub restore_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    pub id: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub user_id: Option<String>,
    pub details: Option<serde_json::Value>,
    pub timestamp: String,
    /// BLAKE3 hash of the previous entry's serialized JSON (for chain verification).
    #[serde(default)]
    pub prev_hash: String,
    /// BLAKE3 hash of this entry (without the entry_hash field itself).
    #[serde(default)]
    pub entry_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FileVersion {
    pub id: String,
    pub file_id: String,
    pub version_number: u32,
    pub hash_blake3: Option<String>,
    pub size_bytes: u64,
    pub snapshot_data: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ShareLink {
    pub id: String,
    pub file_id: String,
    pub token: String,
    pub expires_at: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl ShareLink {
    pub fn with_url(mut self) -> Self {
        self.url = Some(format!("http://localhost:3456/api/shared/{}", self.token));
        self
    }
}

// ---------------------------------------------------------------------------
// Portable Database (`.cybermanju`) types
// ---------------------------------------------------------------------------

/// Cross-platform file relation tracking.
/// Maps a local file to its copies on connected backends.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FileRelation {
    pub id: String,
    pub local_file_id: String,
    pub backend_type: String,
    pub remote_file_id: Option<String>,
    pub remote_path: String,
    pub remote_url: Option<String>,
    pub synced_at: String,
    pub last_verified_at: Option<String>,
    pub status: String, // "active", "deleted", "pending_delete"
}

/// Tracks cross-platform deletion events for propagation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeletionRecord {
    pub id: String,
    pub local_file_id: String,
    pub file_name: String,
    pub deleted_from: String, // platform where deletion originated
    pub deleted_at: String,
    pub deleted_by: Option<String>,
    pub propagated_to: Vec<String>, // platforms where deletion has been applied
    pub pending_platforms: Vec<String>, // platforms still needing deletion
    pub has_compressed_version: bool,
    pub has_preview: bool,
    pub recovery_file_id: Option<String>,
}

/// Recovery entry — references a compressed/preview version stored in the portable DB.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryEntry {
    pub id: String,
    pub original_file_id: String,
    pub original_name: String,
    pub original_mime: Option<String>,
    pub has_compressed: bool,
    pub has_preview: bool,
    pub compressed_hash: Option<String>,
    pub preview_hash: Option<String>,
    pub compressed_size: u64,
    pub preview_size: u64,
    pub stored_at: String,
    pub blob_offset_compressed: Option<u64>,
    pub blob_offset_preview: Option<u64>,
}

/// Header and metadata for a `.cybermanju` portable database file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PortableHeader {
    pub version: String,
    pub created_at: String,
    pub last_modified_at: String,
    pub app_version: String,
    pub db_hash: String,
    pub encryption_algorithm: Option<String>,
    pub compression_algorithm: String,
    pub key_id: Option<String>,
    pub total_files: u64,
    pub total_previews: u64,
    pub total_relations: u64,
    pub total_deletions: u64,
    pub db_size_bytes: u64,
    pub content_store_size: u64,
    pub preview_store_size: u64,
    pub platform_origin: String,
    pub synced_platforms: Vec<String>,
}

/// Entry in the portable DB's content store.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentStoreEntry {
    pub file_id: String,
    pub original_hash: String,
    pub compressed_hash: String,
    pub compression_layer: String,
    pub encrypted: bool,
    pub size_original: u64,
    pub size_compressed: u64,
    pub blob_offset: u64,
    pub blob_length: u64,
    pub mime_type: Option<String>,
}

/// Entry in the portable DB's preview store.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewStoreEntry {
    pub file_id: String,
    pub original_hash: String,
    pub preview_hash: String,
    pub size_bytes: u64,
    pub blob_offset: u64,
    pub blob_length: u64,
    pub width: u32,
    pub height: u32,
    pub mime_type: String, // "image/png", "image/jpeg", "video/webm"
}
