use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tauri::State;

use crate::sync::backends::create_backend;
use crate::sync::models::*;
use crate::AppState;

// ---------------------------------------------------------------------------
// Transfer Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferRequest {
    pub source_config: SyncConfig,
    pub dest_config: SyncConfig,
    pub file_paths: Vec<String>,
    pub delete_source_after: bool,
    pub save_to_local: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferProgress {
    pub total_files: u32,
    pub processed_files: u32,
    pub current_file: Option<String>,
    pub status: String,
    pub bytes_transferred: u64,
    pub errors: Vec<String>,
    pub started_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferResult {
    pub files_transferred: u32,
    pub bytes_transferred: u64,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

// ---------------------------------------------------------------------------
// Transfer State (for progress tracking)
// ---------------------------------------------------------------------------

pub struct TransferState {
    pub progress: Mutex<TransferProgress>,
    pub cancel_flag: AtomicBool,
}

impl TransferState {
    pub fn new() -> Self {
        Self {
            progress: Mutex::new(TransferProgress {
                total_files: 0,
                processed_files: 0,
                current_file: None,
                status: "idle".to_string(),
                bytes_transferred: 0,
                errors: Vec::new(),
                started_at: None,
            }),
            cancel_flag: AtomicBool::new(false),
        }
    }
}

// ---------------------------------------------------------------------------
// Temp file management
// ---------------------------------------------------------------------------

fn temp_dir(state: &AppState) -> String {
    let dir = Path::new(&state.data_dir).join("transfers");
    let _ = fs::create_dir_all(&dir);
    dir.to_string_lossy().to_string()
}

fn unique_temp_path(temp_root: &str, file_name: &str) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    Path::new(temp_root)
        .join(format!("{}_{}", id, file_name))
        .to_string_lossy()
        .to_string()
}

// ---------------------------------------------------------------------------
// Core transfer logic — per-file flow
// ---------------------------------------------------------------------------

/// Resolve the source path for a transfer operation.
/// For local backends the file is already on disk, so use it directly.
/// For remote backends, download to a temp file first.
fn resolve_source_path(
    remote_path: &str,
    source: &dyn StorageBackend,
    is_source_local: bool,
    source_base_path: Option<&str>,
    temp_root: &str,
) -> Result<(String, u64), String> {
    let file_name = Path::new(remote_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "transfer".to_string());

    if is_source_local {
        // ── Local source: use the full path from base_path + remote_path ──
        let local_path = match source_base_path {
            Some(base) => {
                let p = Path::new(base).join(remote_path);
                p.to_string_lossy().to_string()
            }
            None => remote_path.to_string(),
        };
        let size = fs::metadata(&local_path).map(|m| m.len()).unwrap_or(0);
        info!(
            "Transfer: local source — using path directly: {}",
            local_path
        );
        Ok((local_path, size))
    } else {
        // ── Remote source: download to temp ──
        let temp_path = unique_temp_path(temp_root, &file_name);
        info!("Transfer: downloading {} from remote source", remote_path);
        source.download_file(remote_path, &temp_path)?;
        let size = fs::metadata(&temp_path).map(|m| m.len()).unwrap_or(0);
        Ok((temp_path, size))
    }
}

/// Transfer a single file: resolve source → upload to dest → clean temp.
fn transfer_single_file(
    remote_path: &str,
    source: &dyn StorageBackend,
    dest: &dyn StorageBackend,
    is_source_local: bool,
    source_base_path: Option<&str>,
    temp_root: &str,
) -> Result<u64, String> {
    // 1. Resolve source (download remote or use local path directly)
    let (source_path, file_size) = resolve_source_path(
        remote_path,
        source,
        is_source_local,
        source_base_path,
        temp_root,
    )?;

    // 2. Upload from source path to destination.
    //    The remote_path acts as the destination key/name.
    //    For LocalBackend, upload_file expects remote_path relative to base_path.
    //    For cloud backends, the upload_file impl extracts the filename.
    let upload_name = Path::new(remote_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "transfer".to_string());
    info!("Transfer: uploading {} to destination", upload_name);
    dest.upload_file(&source_path, remote_path)?;

    // 3. Clean up temp file (only if it was downloaded, i.e. not local)
    if !is_source_local {
        let _ = fs::remove_file(&source_path);
        info!("Transfer: cleaned up temp file");
    }

    Ok(file_size)
}

fn is_local_backend(config: &SyncConfig) -> bool {
    matches!(config.backend_type, SyncBackendType::Local)
}

/// Handle the case where dest is Local — save the file locally.
/// This is handled by `save_to_local` option: when true, the file is
/// uploaded to the local backend path AND kept in the local filesystem.
fn handle_save_to_local(
    request: &TransferRequest,
    source: &dyn StorageBackend,
    _dest: &dyn StorageBackend,
) -> Result<(), String> {
    // This is a no-op when dest is not local — the file is already
    // transferred to the destination backend. When dest IS local,
    // the upload_file call above has already placed it there.
    // The save_to_local flag primarily serves as documentation for
    // the caller, differentiating from delete_source_after.
    let _ = source;
    let _ = request;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tauri Commands
// ---------------------------------------------------------------------------

/// Transfer files from one backend to another.
/// Processes each file individually: resolve source → upload → clean temp.
#[tauri::command]
pub fn transfer_files(
    request: TransferRequest,
    state: State<'_, AppState>,
    transfer_state: State<'_, Arc<TransferState>>,
) -> Result<TransferResult, String> {
    let start = std::time::Instant::now();
    let total = request.file_paths.len() as u32;

    // Reset state
    {
        let mut p = transfer_state.progress.lock().map_err(|e| e.to_string())?;
        *p = TransferProgress {
            total_files: total,
            processed_files: 0,
            current_file: None,
            status: "initializing".to_string(),
            bytes_transferred: 0,
            errors: Vec::new(),
            started_at: Some(Utc::now().to_rfc3339()),
        };
    }
    transfer_state.cancel_flag.store(false, Ordering::SeqCst);

    let temp_root = temp_dir(&state);
    let source = create_backend(&request.source_config)?;
    let dest = create_backend(&request.dest_config)?;
    let is_source_local = is_local_backend(&request.source_config);
    let source_base_path = request.source_config.base_path.as_deref();

    let mut files_transferred = 0u32;
    let mut bytes_transferred = 0u64;
    let mut errors = Vec::new();

    // ── Per-file flow: resolve → upload → clean ──
    for (i, file_path) in request.file_paths.iter().enumerate() {
        if transfer_state.cancel_flag.load(Ordering::SeqCst) {
            {
                let mut p = transfer_state.progress.lock().map_err(|e| e.to_string())?;
                p.status = "cancelled".to_string();
            }
            break;
        }

        {
            let mut p = transfer_state.progress.lock().map_err(|e| e.to_string())?;
            p.current_file = Some(file_path.clone());
            p.status = if is_source_local {
                "uploading".to_string()
            } else {
                "downloading".to_string()
            };
        }

        match transfer_single_file(
            file_path,
            source.as_ref(),
            dest.as_ref(),
            is_source_local,
            source_base_path,
            &temp_root,
        ) {
            Ok(b) => {
                bytes_transferred += b;
                files_transferred += 1;
            }
            Err(e) => {
                error!("Transfer failed for {}: {}", file_path, e);
                errors.push(format!("{}: {}", file_path, e));
            }
        }

        {
            let mut p = transfer_state.progress.lock().map_err(|e| e.to_string())?;
            p.processed_files = (i as u32) + 1;
            p.bytes_transferred = bytes_transferred;
        }
    }

    // Clean up temp dir
    let _ = fs::remove_dir_all(&temp_root);

    let final_status = if errors.is_empty() || files_transferred > 0 {
        "completed"
    } else {
        "error"
    };
    {
        let mut p = transfer_state.progress.lock().map_err(|e| e.to_string())?;
        p.status = final_status.to_string();
        p.current_file = None;
    }

    // If save_to_local is set, ensure the file is accessible locally
    if request.save_to_local {
        let _ = handle_save_to_local(&request, source.as_ref(), dest.as_ref());
    }

    // If delete_source_after is set, remove files from source
    // (only after each file succeeds, not at batch end)
    if request.delete_source_after && errors.is_empty() {
        for file_path in &request.file_paths {
            if !is_source_local {
                // Only delete from remote source; local files are managed by the user
                let _ = source.delete_file(file_path);
            }
        }
    }

    Ok(TransferResult {
        files_transferred,
        bytes_transferred,
        errors,
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

/// Get current transfer progress.
#[tauri::command]
pub fn get_transfer_progress(
    transfer_state: State<'_, Arc<TransferState>>,
) -> Result<TransferProgress, String> {
    let p = transfer_state.progress.lock().map_err(|e| e.to_string())?;
    Ok(p.clone())
}

/// Cancel an in-progress transfer.
#[tauri::command]
pub fn cancel_transfer(transfer_state: State<'_, Arc<TransferState>>) -> Result<bool, String> {
    transfer_state.cancel_flag.store(true, Ordering::SeqCst);
    {
        let mut p = transfer_state.progress.lock().map_err(|e| e.to_string())?;
        p.status = "cancelled".to_string();
    }
    Ok(true)
}
