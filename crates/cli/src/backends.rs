// Thin re-export of the shared cybermanju-backends crate.
// The CLI wraps the shared factory to preserve Option-based return type.

pub use cybermanju_backends::{
    create_backend as create_backend_internal, transfer_files, GitHubBackend, GitLabBackend,
    GoogleDriveBackend, GooglePhotosBackend, LocalBackend, MegaBackend, TelegramBackend,
};

use cybermanju_types::sync::{StorageBackend, SyncBackendType};

/// Create a backend from its type, token, and JSON config.
/// Returns None if the backend type is unrecognised or creation fails.
pub fn create_backend(
    backend_type: SyncBackendType,
    token: &str,
    config: &serde_json::Value,
) -> Option<Box<dyn StorageBackend>> {
    cybermanju_backends::create_backend(&backend_type, token, config).ok()
}
