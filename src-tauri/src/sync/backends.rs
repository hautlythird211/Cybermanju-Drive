// Cybermanju Drive — Storage Sync Backend Wrapper
// Delegates all backend implementations to the shared cybermanju-backends crate.
// This file only contains the Tauri-specific factory that maps SyncConfig → shared create_backend.

use crate::sync::models::SyncConfig;
use cybermanju_types::sync::{StorageBackend, SyncBackendType};

/// Create a backend from a Tauri SyncConfig.
/// Maps SyncConfig fields to the shared crate's create_backend signature.
pub fn create_backend(config: &SyncConfig) -> Result<Box<dyn StorageBackend>, String> {
    let token = config.token.as_deref().unwrap_or("");

    match config.backend_type {
        SyncBackendType::Local => {
            let base = config
                .base_path
                .as_deref()
                .ok_or("Local backend requires base_path")?;
            cybermanju_backends::create_backend(&config.backend_type, base, &serde_json::json!({}))
        }
        SyncBackendType::GitHub => {
            let cfg = serde_json::json!({
                "repo": config.repo_name,
                "branch": config.branch,
            });
            cybermanju_backends::create_backend(&config.backend_type, token, &cfg)
        }
        SyncBackendType::GitLab => {
            let cfg = serde_json::json!({
                "project": config.repo_name,
                "branch": config.branch,
                "base_url": config.base_path,
            });
            cybermanju_backends::create_backend(&config.backend_type, token, &cfg)
        }
        SyncBackendType::GoogleDrive => {
            let cfg = serde_json::json!({ "folder_id": config.folder_id });
            cybermanju_backends::create_backend(&config.backend_type, token, &cfg)
        }
        SyncBackendType::GooglePhotos => {
            let cfg = serde_json::json!({ "album_id": config.album_id });
            cybermanju_backends::create_backend(&config.backend_type, token, &cfg)
        }
        SyncBackendType::Telegram => {
            let cfg = serde_json::json!({ "chat_id": config.chat_id });
            cybermanju_backends::create_backend(&config.backend_type, token, &cfg)
        }
        SyncBackendType::Mega => {
            let t = config
                .token
                .as_deref()
                .ok_or("Mega backend requires token in 'email|password' format")?;
            let parts: Vec<&str> = t.splitn(2, '|').collect();
            if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
                return Err("Mega backend requires token in 'email|password' format".to_string());
            }
            let cfg = serde_json::json!({ "password": parts[1] });
            cybermanju_backends::create_backend(&config.backend_type, parts[0], &cfg)
        }
    }
}

// Re-export all shared backend structs for any code that references them directly.
pub use cybermanju_backends::{
    transfer_files, GitHubBackend, GitLabBackend, GoogleDriveBackend, GooglePhotosBackend,
    LocalBackend, MegaBackend, TelegramBackend,
};
