mod util;

pub mod codeberg;
pub mod git_lfs;
pub mod gitea;
pub mod github;
pub mod gitlab;
pub mod google_drive;
pub mod google_photos;
pub mod local;
pub mod mega;
pub mod repo_layout;
pub mod telegram;
pub mod transfer;

pub use codeberg::CodebergBackend;
pub use gitea::GiteaBackend;
pub use github::GitHubBackend;
pub use gitlab::GitLabBackend;
pub use google_drive::GoogleDriveBackend;
pub use google_photos::GooglePhotosBackend;
pub use local::LocalBackend;
pub use mega::MegaBackend;
pub use telegram::TelegramBackend;
pub use transfer::transfer_files;

use cybermanju_types::sync::{RepoLayout, StorageBackend, SyncBackendType};

pub fn create_backend(
    backend_type: &SyncBackendType,
    token: &str,
    config: &serde_json::Value,
) -> Result<Box<dyn StorageBackend>, String> {
    let use_lfs = config
        .get("use_git_lfs")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let layout = config
        .get("repo_layout")
        .and_then(|v| v.as_str())
        .map(RepoLayout::from_str)
        .unwrap_or(RepoLayout::Flat);
    let lfs_repo = config
        .get("lfs_repo")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

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
            Ok(Box::new(
                GitHubBackend::new(token, repo, branch)
                    .with_lfs(use_lfs)
                    .with_layout(layout)
                    .with_lfs_repo(lfs_repo),
            ))
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
            Ok(Box::new(
                GitLabBackend::new(token, project, branch, base_url).with_lfs(use_lfs),
            ))
        }
        SyncBackendType::Codeberg => {
            let repo = config
                .get("repo")
                .and_then(|v| v.as_str())
                .unwrap_or("user/repo");
            let branch = config
                .get("branch")
                .and_then(|v| v.as_str())
                .unwrap_or("main");
            Ok(Box::new(CodebergBackend::new(token, repo, branch)))
        }
        SyncBackendType::Gitea => {
            let repo = config
                .get("repo")
                .and_then(|v| v.as_str())
                .unwrap_or("user/repo");
            let branch = config
                .get("branch")
                .and_then(|v| v.as_str())
                .unwrap_or("main");
            let base_url = config.get("base_url").and_then(|v| v.as_str());
            Ok(Box::new(GiteaBackend::new(token, repo, branch, base_url)))
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
            MegaBackend::new_with_email_password(token, password)
                .map(|b| Box::new(b) as Box<dyn StorageBackend>)
        }
    }
}
