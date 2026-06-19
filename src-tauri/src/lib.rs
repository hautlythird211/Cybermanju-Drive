// Cybermanju Drive — Core Library
// Orchestrates redb, ML-KEM PQC (pqcrypto-mlkem), Tantivy, Tree-sitter, triple compression, face clustering

use std::path::{Path, PathBuf};

pub mod commands;
pub mod compression;
pub mod crypto;
pub mod db;
pub mod faces; // ML module: detect_faces_in_file, embedding_distance, cluster_embeddings
pub mod preview;
pub mod search;
pub mod sync;
pub mod transfer;
pub mod tree_sitter; // parse_file, get_symbols (tauri commands)
pub mod web_dashboard;

use commands::faces as face_cmd;
use commands::sync as sync_cmd;
use commands::{
    accounts, audit, batch, collections, dashboard, encryption, files, import as import_cmd, map,
    portable_db, search as search_cmd, share, trash, users, versions,
};
use db::Database;
use std::sync::{Arc, RwLock};

pub struct AppState {
    pub db: RwLock<Database>,
    pub db_path: String,
    pub data_dir: String,
    pub tantivy_index: RwLock<search::SearchIndex>,
    pub compression: compression::TripleCompressor,
    pub hmac_secret: [u8; 32],
}

/// Resolve a cross-platform app data directory, creating it if necessary.
fn app_data_dir() -> PathBuf {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cybermanju-drive");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Write a message to the crash log file in the app data directory.
fn write_crash_log(data_dir: &Path, msg: &str) {
    let path = data_dir.join("crash.log");
    let _ = std::fs::write(&path, msg);
}

/// Fatal startup error — log it, write crash file, then exit.
fn fatal(data_dir: &Path, msg: &str) -> ! {
    tracing::error!("{}", msg);
    write_crash_log(data_dir, &format!("FATAL: {}\n", msg));
    std::process::exit(1);
}

// WebDashboard now handles its own shutdown (Drop impl with signal channel + thread join).

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Resolve cross-platform app data directory first (needed for crash log path)
    let data_dir = app_data_dir();
    let data_dir_str = data_dir.to_string_lossy().to_string();

    // ── Crash log & panic hook (writes to file, useful when no console) ──
    write_crash_log(&data_dir, "Cybermanju Drive — startup in progress...\n");
    let crash_path = data_dir.join("crash.log");
    let cp = crash_path.clone();
    std::panic::set_hook(Box::new(move |info| {
        let msg = format!(
            "Cybermanju Drive v{} crashed!\n\
             ============================\n\
             Time: {}\n\
             Panic: {}\n\
             Location: {}\n\
             \n\
             Please report this issue at:\n\
             https://github.com/cybermanju/cybermanju-drive/issues\n\
             \n\
             Attach this file for debugging.\n",
            env!("CARGO_PKG_VERSION"),
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            info,
            info.location().map(|l| l.to_string()).unwrap_or_default(),
        );
        let _ = std::fs::write(&cp, &msg);
    }));

    // Initialize tracing subscriber (replaces env_logger)
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    tracing::info!("Cybermanju Drive starting...");
    tracing::info!("App data directory: {}", data_dir_str);

    // Build paths under the data directory
    let db_path_str = data_dir.join("cybermanju.db").to_string_lossy().to_string();
    let portable_db_path_str = data_dir.join(".cybermanju").to_string_lossy().to_string();
    let search_index_path_str = data_dir.join("tantivy_index").to_string_lossy().to_string();

    // Initialize redb database
    let db = match Database::new(&db_path_str) {
        Ok(d) => d,
        Err(e) => fatal(
            &data_dir,
            &format!("Failed to initialize redb database: {}", e),
        ),
    };
    tracing::info!("redb database initialized");

    // Initialize .cybermanju portable database
    let platform = if std::env::var("DOCKER_MODE").is_ok() {
        "docker"
    } else {
        "local"
    };
    match cybermanju_portable_db::PortableDatabase::open_or_create(&portable_db_path_str, platform)
    {
        Ok(pdb) => {
            tracing::info!(
                ".cybermanju portable database ready at {} ({} files, {} relations)",
                portable_db_path_str,
                pdb.header().total_files,
                pdb.header().total_relations
            );
            // Store the path in the redb database for cross-reference
            let _ = db.set_portable_meta("portable_db_path", &portable_db_path_str);
            let _ = db.set_portable_meta("portable_db_origin", platform);
        }
        Err(e) => {
            tracing::error!("Failed to initialize .cybermanju portable database: {}", e);
        }
    }

    // Initialize Tantivy full-text search index
    let tantivy_index = match search::SearchIndex::new(&search_index_path_str) {
        Ok(i) => i,
        Err(e) => fatal(&data_dir, &format!("Failed to initialize Tantivy: {}", e)),
    };
    tracing::info!("Tantivy search index ready");

    // Initialize triple-layer compressor
    let compressor = compression::TripleCompressor::new();

    // Initialize HMAC secret for secure session tokens
    let mut hmac_secret = [0u8; 32];
    use rand_core::{OsRng, RngCore};
    OsRng.fill_bytes(&mut hmac_secret);

    let state = AppState {
        db: RwLock::new(db),
        db_path: db_path_str.clone(),
        data_dir: data_dir_str,
        tantivy_index: RwLock::new(tantivy_index),
        compression: compressor,
        hmac_secret,
    };

    // Dashboard state for connection tracking and lifecycle
    let dashboard_state = Arc::new(dashboard::DashboardState::new());

    // Sync state for progress tracking and cancellation
    let sync_state = Arc::new(sync_cmd::SyncState::new());

    // Transfer state for progress tracking and cancellation
    let transfer_state = Arc::new(transfer::TransferState::new());

    // ─── Start Web Dashboard ────────────────────────────────────────────
    // Bind to 0.0.0.0 when DOCKER_MODE env var is set (NAS/container access).
    // Otherwise bind to 127.0.0.1 for local-only security (Tauri desktop).
    let dashboard_bind = if std::env::var("DOCKER_MODE").is_ok() {
        tracing::info!("DOCKER_MODE detected — binding web dashboard to 0.0.0.0");
        "0.0.0.0"
    } else {
        "127.0.0.1"
    };
    let dashboard = std::sync::Arc::new(web_dashboard::WebDashboard::new_with_bind_addr(
        web_dashboard::DEFAULT_PORT,
        &db_path_str,
        dashboard_bind,
    ));
    match dashboard.start() {
        Ok(()) => tracing::info!(
            "Web Dashboard started on {}:{} (JWT auth)",
            dashboard_bind,
            web_dashboard::DEFAULT_PORT
        ),
        Err(e) => tracing::error!("Failed to start Web Dashboard: {}", e),
    }
    // dashboard.stop() is called explicitly below after Tauri exits,
    // ensuring the accept thread is joined from the MAIN thread (not from
    // the accept thread's own Drop, which would self-deadlock).

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_conduit::init().build())
        .plugin(tauri_plugin_clipboard_next::init())
        .plugin(tauri_plugin_serialplugin::init());

    // Windows-only: snap layout for frameless windows
    #[cfg(target_os = "windows")]
    let builder = builder.plugin(tauri_plugin_snap_layout::init());

    builder
        .manage(state)
        .manage(dashboard_state)
        .manage(sync_state)
        .manage(transfer_state)
        .invoke_handler(tauri::generate_handler![
            // File operations
            files::list_files,
            files::get_file,
            files::create_folder,
            files::delete_file,
            files::rename_file,
            files::duplicate_file_context,
            files::move_file,
            files::get_preview,
            // Search
            search_cmd::search_files,
            search_cmd::search_files_paginated,
            search_cmd::suggest,
            // Encryption
            encryption::encrypt_file,
            encryption::decrypt_file,
            encryption::get_encryption_status,
            encryption::generate_keypair,
            encryption::list_keys,
            // Compression
            commands::compression::compress_file,
            commands::compression::decompress_file,
            commands::compression::get_compression_stats,
            // Collections
            collections::list_collections,
            collections::create_collection,
            collections::add_to_collection,
            collections::remove_from_collection,
            // Face grouping (commands layer — delegates to crate::faces for ML)
            face_cmd::detect_faces,
            face_cmd::detect_faces_batch_cmd,
            face_cmd::recluster_faces,
            face_cmd::rename_face_group,
            face_cmd::merge_face_groups,
            face_cmd::delete_face_group,
            face_cmd::find_similar_faces,
            face_cmd::list_face_groups,
            face_cmd::get_group_files,
            // Map / GPS
            map::get_geo_files,
            map::extract_exif_gps,
            // Accounts
            accounts::list_accounts,
            accounts::create_account,
            accounts::switch_account,
            accounts::delete_account,
            // Tree-sitter code intelligence
            tree_sitter::parse_file,
            tree_sitter::get_symbols,
            // Loose groups
            files::create_loose_group,
            files::add_to_loose_group,
            files::list_loose_groups,
            // User management & permissions
            users::register_user,
            users::delete_user,
            users::update_user_role,
            users::authenticate_user,
            users::list_users,
            users::set_file_permission,
            users::grant_file_permission,
            users::revoke_file_permission,
            users::verify_file_access,
            users::get_file_permissions,
            users::verify_token,
            // Dashboard
            dashboard::dashboard_status,
            dashboard::start_dashboard,
            dashboard::stop_dashboard,
            // Sync
            sync_cmd::list_sync_configs,
            sync_cmd::create_sync_config,
            sync_cmd::delete_sync_config,
            sync_cmd::start_sync,
            sync_cmd::get_sync_progress,
            sync_cmd::test_sync_connection,
            sync_cmd::cancel_sync,
            sync_cmd::list_remote_files,
            // File import / upload
            import_cmd::import_file,
            import_cmd::import_from_url,
            import_cmd::scan_directory,
            import_cmd::upload_file,
            import_cmd::rebuild_search_index,
            // Share links
            share::generate_share_link,
            share::get_shared_file,
            share::list_share_links,
            // Trash / recycle bin
            trash::list_trash,
            trash::restore_from_trash,
            trash::empty_trash,
            trash::delete_from_trash,
            // Audit log
            audit::get_audit_log,
            // Batch operations
            batch::batch_delete,
            batch::batch_encrypt,
            batch::batch_compress,
            // File versioning
            versions::list_file_versions,
            versions::create_file_version,
            versions::revert_file_version,
            versions::snapshot_all_versions,
            // Parent index rebuild
            files::rebuild_parent_index,
            // Backend-aware deletion
            files::delete_files_from_backends,
            // Metadata-only deletion
            files::delete_file_metadata_only,
            // Transfer
            transfer::transfer_files,
            transfer::get_transfer_progress,
            transfer::cancel_transfer,
            // Diagnostics / crash log
            commands::diagnostics::get_crash_log,
            commands::diagnostics::clear_crash_log,
            commands::diagnostics::get_app_log,
            // Duplicate detection
            commands::duplicates::find_duplicates,
            // Portable Database (`.cybermanju`)
            portable_db::init_portable_db,
            portable_db::get_portable_db_header,
            portable_db::sync_portable_db,
            portable_db::record_file_relation,
            portable_db::get_file_relations,
            portable_db::list_all_relations,
            portable_db::record_deletion,
            portable_db::list_pending_deletions,
            portable_db::list_all_deletions,
            portable_db::store_compressed_for_recovery,
            portable_db::store_preview_for_recovery,
            portable_db::list_recoverable_files,
            portable_db::recover_file,
            portable_db::get_recovery_preview,
            portable_db::repack_portable_db,
            portable_db::mark_deletion_propagated,
            portable_db::get_portable_db_meta,
            portable_db::delete_recovery_entry,
            // KV Store (general-purpose key-value persistence)
            commands::kv::kv_set,
            commands::kv::kv_get,
            // System info
            commands::system_info::get_system_info,
            // Web browser
            commands::web::web_search,
            commands::web::fetch_page,
        ])
        .run(tauri::generate_context!())
        .expect("Fatal error while running Cybermanju Drive — see logs above");

    // ─── Clean shutdown: stop the dashboard before dropping the Arc ──
    dashboard.stop();
    log::info!("Web Dashboard shut down cleanly");
}
