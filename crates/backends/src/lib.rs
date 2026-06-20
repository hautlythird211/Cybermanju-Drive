mod util;

pub mod local;
pub mod github;
pub mod gitlab;
pub mod google_drive;
pub mod google_photos;
pub mod telegram;
pub mod mega;
pub mod transfer;

pub use local::LocalBackend;
pub use github::GitHubBackend;
pub use gitlab::GitLabBackend;
pub use google_drive::GoogleDriveBackend;
pub use google_photos::GooglePhotosBackend;
pub use telegram::TelegramBackend;
pub use mega::MegaBackend;
pub use transfer::transfer_files;

use cybermanju_types::sync::{StorageBackend, SyncBackendType};

pub fn create_backend(
    backend_type: &SyncBackendType,
    token: &str,
    config: &serde_json::Value,
) -> Result<Box<dyn StorageBackend>, String> {
    match backend_type {
        SyncBackendType::Local => Ok(Box::new(LocalBackend::new(token))),
        SyncBackendType::GitHub => {
            let repo = config
                .get("repo")
                .and_then(|v| v.as_str())
                .unwrap_or("user/repo");
            let branch = config
                .get("branch")
                .and_then(|v| v.as_str())
                .unwrap_or("main");
            Ok(Box::new(GitHubBackend::new(token, repo, branch)))
        }
        SyncBackendType::GitLab => {
            let project = config
                .get("project")
                .and_then(|v| v.as_str())
                .unwrap_or("0");
            let branch = config
                .get("branch")
                .and_then(|v| v.as_str())
                .unwrap_or("main");
            let base_url = config.get("base_url").and_then(|v| v.as_str());
            Ok(Box::new(GitLabBackend::new(token, project, branch, base_url)))
        }
        SyncBackendType::GoogleDrive => {
            let folder = config.get("folder_id").and_then(|v| v.as_str());
            Ok(Box::new(GoogleDriveBackend::new(token, folder)))
        }
        SyncBackendType::GooglePhotos => {
            let album = config.get("album_id").and_then(|v| v.as_str());
            Ok(Box::new(GooglePhotosBackend::new(token, album)))
        }
        SyncBackendType::Telegram => {
            let chat = config
                .get("chat_id")
                .and_then(|v| v.as_str())
                .unwrap_or("0");
            Ok(Box::new(TelegramBackend::new(token, chat)))
        }
        SyncBackendType::Mega => {
            let password = config
                .get("password")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            MegaBackend::new(token, password).map(|b| Box::new(b) as Box<dyn StorageBackend>)
        }
    }
}
