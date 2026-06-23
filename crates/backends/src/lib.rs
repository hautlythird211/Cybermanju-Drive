mod util;

pub mod activitypub;
pub mod codeberg;
pub mod git_lfs;
pub mod gitea;
pub mod github;
pub mod gitlab;
pub mod google_drive;
pub mod google_photos;
pub mod iroh;
pub mod lan;
pub mod local;
pub mod mega;
pub mod nostr;
pub mod rclone;
pub mod relay_updater;
pub mod repo_layout;
pub mod telegram;
pub mod torrent;
pub mod transfer;

pub use activitypub::{ActivityPubBackend, ActivityPubShare};
pub use codeberg::CodebergBackend;
pub use gitea::GiteaBackend;
pub use github::GitHubBackend;
pub use gitlab::GitLabBackend;
pub use google_drive::GoogleDriveBackend;
pub use google_photos::GooglePhotosBackend;
pub use iroh::IrohBackend;
pub use lan::LanBackend;
pub use local::LocalBackend;
pub use mega::MegaBackend;
pub use nostr::NostrBackend;
pub use rclone::RcloneBackend;
pub use relay_updater::RelayIpList;
pub use telegram::TelegramBackend;
pub use torrent::TorrentBackend;
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
        .map(RepoLayout::parse_str)
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
        SyncBackendType::Nostr => {
            let relays: Vec<String> = config
                .get("relays")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let nip96_host = config
                .get("nip96_host")
                .and_then(|v| v.as_str())
                .map(String::from);
            let private_key = token.as_bytes().to_vec();
            Ok(Box::new(NostrBackend::new(private_key, relays, nip96_host)))
        }
        SyncBackendType::Iroh => {
            let gateway_url = config
                .get("gateway_url")
                .and_then(|v| v.as_str())
                .map(String::from);
            Ok(Box::new(IrohBackend::new(gateway_url)))
        }
        SyncBackendType::Torrent => {
            let save_dir = config
                .get("save_dir")
                .and_then(|v| v.as_str())
                .unwrap_or("/tmp/torrents");
            let seed_port = config
                .get("seed_port")
                .and_then(|v| v.as_u64())
                .unwrap_or(6881) as u16;
            let tracker_url = config
                .get("tracker_url")
                .and_then(|v| v.as_str())
                .map(String::from);
            Ok(Box::new(TorrentBackend::new(
                std::path::PathBuf::from(save_dir),
                seed_port,
                tracker_url,
            )))
        }
        SyncBackendType::ActivityPub => {
            let collection_id = config
                .get("collection_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let actor_id = config
                .get("actor_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let endpoint = config
                .get("endpoint")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let share = ActivityPubShare::new(collection_id, actor_id, endpoint, token.to_string());
            Ok(Box::new(ActivityPubBackend::new(share)))
        }
        SyncBackendType::Lan => {
            let service_name = config
                .get("service_name")
                .and_then(|v| v.as_str())
                .unwrap_or("_cybermanju._tcp")
                .to_string();
            let device_key = token.as_bytes().to_vec();
            Ok(Box::new(LanBackend::new(service_name, device_key)))
        }
        SyncBackendType::Rclone => {
            let remote_name = config
                .get("remote_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let rclone_path = config
                .get("rclone_path")
                .and_then(|v| v.as_str())
                .unwrap_or("rclone");
            Ok(Box::new(RcloneBackend::new(
                remote_name,
                std::path::PathBuf::from(rclone_path),
            )))
        }
    }
}
