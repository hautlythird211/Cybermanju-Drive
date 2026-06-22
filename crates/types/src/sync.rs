use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum SyncBackendType {
    Local,
    GitHub,
    GitLab,
    Codeberg,
    Gitea,
    GoogleDrive,
    GooglePhotos,
    Telegram,
    Mega,
    Nostr,
    Iroh,
    Torrent,
    ActivityPub,
    Lan,
    Rclone,
}

impl std::fmt::Display for SyncBackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "local"),
            Self::GitHub => write!(f, "gitHub"),
            Self::GitLab => write!(f, "gitLab"),
            Self::Codeberg => write!(f, "codeberg"),
            Self::Gitea => write!(f, "gitea"),
            Self::GoogleDrive => write!(f, "googleDrive"),
            Self::GooglePhotos => write!(f, "googlePhotos"),
            Self::Telegram => write!(f, "telegram"),
            Self::Mega => write!(f, "mega"),
            Self::Nostr => write!(f, "nostr"),
            Self::Iroh => write!(f, "iroh"),
            Self::Torrent => write!(f, "torrent"),
            Self::ActivityPub => write!(f, "activityPub"),
            Self::Lan => write!(f, "lan"),
            Self::Rclone => write!(f, "rclone"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "idle"),
            Self::Scanning => write!(f, "scanning"),
            Self::Compressing => write!(f, "compressing"),
            Self::Uploading => write!(f, "uploading"),
            Self::Linking => write!(f, "linking"),
            Self::Cleaning => write!(f, "cleaning"),
            Self::Error => write!(f, "error"),
            Self::Done => write!(f, "done"),
            Self::Syncing => write!(f, "syncing"),
            Self::Completed => write!(f, "completed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudAccount {
    pub id: String,
    pub name: String,
    pub backend_type: SyncBackendType,
    pub token: Option<String>,
    pub oauth_credentials: Option<OAuthCredentials>,
    pub config: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthCredentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<u64>,
    pub client_id: String,
    pub client_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncFile {
    pub id: String,
    pub original_path: String,
    pub compressed_path: Option<String>,
    pub preview_path: Option<String>,
    pub remote_url: Option<String>,
    pub size_bytes: u64,
    pub compressed_size_bytes: Option<u64>,
    pub hash_blake3: Option<String>,
    pub backend_type: SyncBackendType,
    pub synced_at: Option<String>,
    pub status: SyncStatus,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncConfig {
    pub id: String,
    pub backend_type: SyncBackendType,
    pub enabled: bool,
    pub account_id: Option<String>,
    pub name: Option<String>,
    pub base_path: Option<String>,
    pub repo_name: Option<String>,
    pub branch: Option<String>,
    pub token: Option<String>,
    pub folder_id: Option<String>,
    pub album_id: Option<String>,
    pub chat_id: Option<String>,
    pub auto_sync: bool,
    pub compress_before_upload: bool,
    pub create_previews: bool,
    pub delete_raw_after_sync: bool,
    pub max_concurrent_uploads: u32,
    /// Use Git LFS for large file storage
    pub use_git_lfs: bool,
    /// Separate LFS blob repo (e.g. "owner/blobs-repo") — if empty, uses same repo
    pub lfs_repo: Option<String>,
    /// Repo layout: "flat" (default), "sharded" (hash-prefix dirs), "split" (separate meta+blob repos)
    pub repo_layout: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncProgress {
    pub total_files: u32,
    pub processed_files: u32,
    pub current_file: Option<String>,
    pub status: SyncStatus,
    pub bytes_uploaded: u64,
    pub errors: Vec<String>,
    pub started_at: Option<String>,
    pub estimated_remaining_seconds: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub files_synced: u32,
    pub bytes_uploaded: u64,
    pub bytes_saved_by_compression: u64,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteFile {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub modified_at: String,
    pub url: String,
}

// ---------------------------------------------------------------------------
// Git LFS types
// ---------------------------------------------------------------------------

/// Git LFS batch API request body.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LfsBatchRequest {
    pub operation: String,
    pub transfers: Vec<String>,
    pub objects: Vec<LfsObject>,
    pub hash_algo: Option<String>,
}

/// An object reference in LFS operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LfsObject {
    pub oid: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authenticated: Option<bool>,
}

/// Single object response from LFS batch API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LfsObjectResponse {
    pub oid: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<LfsActions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<LfsError>,
}

/// Upload/download/verify actions for an LFS object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LfsActions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download: Option<LfsAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upload: Option<LfsAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify: Option<LfsAction>,
}

/// A single LFS action (upload/download/verify) with URL and headers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LfsAction {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<std::collections::HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// LFS batch API response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LfsBatchResponse {
    pub transfer: Option<String>,
    pub objects: Vec<LfsObjectResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash_algo: Option<String>,
}

/// LFS error detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LfsError {
    pub code: i32,
    pub message: String,
}

/// Git LFS pointer file content (the small text file that replaces the large file in Git).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LfsPointer {
    pub version: String, // "https://git-lfs.github.com/spec/v1"
    pub oid: String,     // SHA-256 hex of the large file
    pub size: u64,       // Size in bytes
}

impl std::fmt::Display for LfsPointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "version {}\noid sha256:{}\nsize {}\n",
            self.version, self.oid, self.size
        )
    }
}

impl LfsPointer {
    pub fn from_string(s: &str) -> Result<Self, String> {
        let mut version = String::new();
        let mut oid = String::new();
        let mut size = 0u64;
        for line in s.lines() {
            if let Some(v) = line.strip_prefix("version ") {
                version = v.to_string();
            } else if let Some(v) = line.strip_prefix("oid sha256:") {
                oid = v.to_string();
            } else if let Some(v) = line.strip_prefix("size ") {
                size = v.parse().map_err(|e| format!("invalid size: {}", e))?;
            }
        }
        if version.is_empty() || oid.is_empty() || size == 0 {
            return Err("Invalid LFS pointer".to_string());
        }
        Ok(Self { version, oid, size })
    }

    /// Check if a file is an LFS pointer (starts with "version ").
    pub fn is_lfs_pointer(data: &[u8]) -> bool {
        data.starts_with(b"version ")
    }
}

// ---------------------------------------------------------------------------
// Repo layout types
// ---------------------------------------------------------------------------

/// Describes how the .cybermanju sync structures the remote repository.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RepoLayout {
    /// Flat structure: all files in the repo root /cybermanju_sync/
    Flat,
    /// Sharded by BLAKE3 hash prefix: blobs/ab/cd/abcdef...cyb3
    Sharded,
    /// Split repos: meta in main repo, blobs in a separate LFS blob repo
    Split,
}

impl std::fmt::Display for RepoLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Flat => write!(f, "flat"),
            Self::Sharded => write!(f, "sharded"),
            Self::Split => write!(f, "split"),
        }
    }
}

impl RepoLayout {
    pub fn parse_str(s: &str) -> Self {
        match s {
            "sharded" => Self::Sharded,
            "split" => Self::Split,
            _ => Self::Flat,
        }
    }
}

// ---------------------------------------------------------------------------
// StorageBackend trait
// ---------------------------------------------------------------------------

pub trait StorageBackend: Send + Sync {
    fn name(&self) -> &str;
    fn backend_type(&self) -> SyncBackendType;
    fn upload_file(&self, local_path: &str, remote_path: &str) -> Result<String, String>;
    fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String>;
    fn delete_file(&self, remote_path: &str) -> Result<(), String>;
    fn list_files(&self, prefix: &str) -> Result<Vec<RemoteFile>, String>;
    fn get_file_url(&self, remote_path: &str) -> Result<String, String>;
    fn test_connection(&self) -> Result<bool, String>;
}
