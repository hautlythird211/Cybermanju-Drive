use crate::db::schema::{DeletionRecord, FileRelation, PortableHeader, RecoveryEntry};
use crate::sync::backends::create_backend;
use crate::AppState;
use redb::ReadableTable;
use tauri::State;

#[tauri::command]
pub fn init_portable_db(
    path: String,
    platform_origin: String,
    state: State<'_, AppState>,
) -> Result<PortableHeader, String> {
    let pdb = cybermanju_portable_db::PortableDatabase::open_or_create(&path, &platform_origin)
        .map_err(|e| format!("init portable db: {}", e))?;
    let db = state.db.read().map_err(|e| e.to_string())?;
    db.set_portable_meta("portable_db_path", &path)
        .and(db.set_portable_meta("portable_db_origin", &platform_origin))
        .map_err(|e| e.to_string())?;
    Ok(pdb.header().clone())
}

#[tauri::command]
pub fn get_portable_db_header(path: String) -> Result<PortableHeader, String> {
    let pdb = cybermanju_portable_db::PortableDatabase::open(&path).map_err(|e| e.to_string())?;
    Ok(pdb.header().clone())
}

/// Sync `.cybermanju` to every enabled backend.
/// Also propagates any pending deletions first.
#[tauri::command]
pub fn sync_portable_db(state: State<'_, AppState>) -> Result<Vec<(String, String)>, String> {
    let db = state.db.read().map_err(|e| e.to_string())?;
    let pdb_path = db
        .get_portable_meta("portable_db_path")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "portable db not initialized".to_string())?;
    let tx = db.begin_read().map_err(|e| e.to_string())?;
    let table = tx
        .open_table(crate::db::Database::get_sync_configs_table())
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for entry in table.iter().map_err(|e| e.to_string())? {
        let (_, val) = entry.map_err(|e| e.to_string())?;
        let config: crate::sync::models::SyncConfig =
            serde_json::from_str(val.value()).map_err(|e| e.to_string())?;
        if !config.enabled {
            continue;
        }
        let backend = create_backend(&config)?;
        let pdb =
            cybermanju_portable_db::PortableDatabase::open(&pdb_path).map_err(|e| e.to_string())?;
        // propagate pending deletions for this backend
        let pending = db.list_pending_deletions().map_err(|e| e.to_string())?;
        for rec in &pending {
            for plat in &rec.pending_platforms {
                if plat == &config.backend_type.to_string() {
                    if let Ok(rels) = db.get_file_relations_for_local(&rec.local_file_id) {
                        if let Some(rel) = rels
                            .iter()
                            .find(|r| r.backend_type == *plat && r.status == "active")
                        {
                            let _ = backend.delete_file(&rel.remote_path).and_then(|_| {
                                db.mark_deletion_propagated(
                                    &rec.id,
                                    &backend.backend_type().to_string(),
                                )
                                .map_err(|e| e.to_string())
                            });
                        }
                    }
                }
            }
        }
        // now upload .cybermanju
        match backend.upload_file(pdb.path().to_str().unwrap_or(".cybermanju"), ".cybermanju") {
            Ok(url) => results.push((config.backend_type.to_string(), url)),
            Err(e) => log::error!("sync .cybermanju to {} failed: {}", config.backend_type, e),
        }
    }
    Ok(results)
}

#[tauri::command]
pub fn record_file_relation(
    local_file_id: String,
    backend_type: String,
    remote_path: String,
    remote_url: Option<String>,
    remote_file_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<FileRelation, String> {
    let db = state.db.write().map_err(|e| e.to_string())?;
    cybermanju_portable_db::PortableDatabase::record_file_relation(
        &db,
        &local_file_id,
        &backend_type,
        &remote_path,
        remote_url.as_deref(),
        remote_file_id.as_deref(),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_file_relations(
    local_file_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<FileRelation>, String> {
    let db = state.db.read().map_err(|e| e.to_string())?;
    db.get_file_relations_for_local(&local_file_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_all_relations(state: State<'_, AppState>) -> Result<Vec<FileRelation>, String> {
    let db = state.db.read().map_err(|e| e.to_string())?;
    db.list_all_file_relations().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn record_deletion(
    local_file_id: String,
    file_name: String,
    deleted_from: String,
    state: State<'_, AppState>,
) -> Result<DeletionRecord, String> {
    let db = state.db.write().map_err(|e| e.to_string())?;
    let configs = crate::commands::sync::list_sync_configs_inner(&db)?;
    let platforms: Vec<String> = configs
        .iter()
        .filter(|c| c.enabled)
        .map(|c| c.backend_type.to_string())
        .collect();
    cybermanju_portable_db::PortableDatabase::record_deletion(
        &db,
        &local_file_id,
        &file_name,
        &deleted_from,
        &platforms,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_pending_deletions(state: State<'_, AppState>) -> Result<Vec<DeletionRecord>, String> {
    let db = state.db.read().map_err(|e| e.to_string())?;
    db.list_pending_deletions().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_all_deletions(state: State<'_, AppState>) -> Result<Vec<DeletionRecord>, String> {
    let db = state.db.read().map_err(|e| e.to_string())?;
    db.list_all_deletion_records().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn store_compressed_for_recovery(
    file_id: String,
    file_name: String,
    file_path: String,
    state: State<'_, AppState>,
) -> Result<RecoveryEntry, String> {
    let db = state.db.read().map_err(|e| e.to_string())?;
    let pdb_path = db
        .get_portable_meta("portable_db_path")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "portable db not initialized".to_string())?;
    drop(db);

    let data = std::fs::read(&file_path).map_err(|e| format!("read file: {}", e))?;
    let mime = infer::get_from_path(&file_path)
        .ok()
        .flatten()
        .map(|t| t.mime_type().to_string());

    let pdb =
        cybermanju_portable_db::PortableDatabase::open(&pdb_path).map_err(|e| e.to_string())?;
    let db = state.db.write().map_err(|e| e.to_string())?;
    pdb.store_compressed_content(&db, &file_id, &data, &file_name, mime.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn store_preview_for_recovery(
    file_id: String,
    preview_path: String,
    width: u32,
    height: u32,
    state: State<'_, AppState>,
) -> Result<RecoveryEntry, String> {
    let db = state.db.read().map_err(|e| e.to_string())?;
    let pdb_path = db
        .get_portable_meta("portable_db_path")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "portable db not initialized".to_string())?;
    drop(db);

    let data = std::fs::read(&preview_path).map_err(|e| format!("read preview: {}", e))?;
    let pdb =
        cybermanju_portable_db::PortableDatabase::open(&pdb_path).map_err(|e| e.to_string())?;
    let db = state.db.write().map_err(|e| e.to_string())?;
    pdb.store_preview(&db, &file_id, &data, width, height)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_recoverable_files(state: State<'_, AppState>) -> Result<Vec<RecoveryEntry>, String> {
    let db = state.db.read().map_err(|e| e.to_string())?;
    cybermanju_portable_db::PortableDatabase::list_recoverable_files(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn recover_file(
    file_id: String,
    output_path: String,
    state: State<'_, AppState>,
) -> Result<u64, String> {
    let db = state.db.read().map_err(|e| e.to_string())?;
    let pdb_path = db
        .get_portable_meta("portable_db_path")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "portable db not initialized".to_string())?;
    drop(db);

    let pdb =
        cybermanju_portable_db::PortableDatabase::open(&pdb_path).map_err(|e| e.to_string())?;
    let db = state.db.write().map_err(|e| e.to_string())?;
    match pdb
        .get_recoverable_data(&db, &file_id)
        .map_err(|e| e.to_string())?
    {
        Some((data, _)) => {
            std::fs::write(&output_path, &data).map_err(|e| format!("write: {}", e))?;
            Ok(data.len() as u64)
        }
        None => Err("no recoverable data".to_string()),
    }
}

#[tauri::command]
pub fn get_recovery_preview(
    file_id: String,
    state: State<'_, AppState>,
) -> Result<Option<Vec<u8>>, String> {
    let db = state.db.read().map_err(|e| e.to_string())?;
    let pdb_path = db
        .get_portable_meta("portable_db_path")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "portable db not initialized".to_string())?;
    drop(db);

    let pdb =
        cybermanju_portable_db::PortableDatabase::open(&pdb_path).map_err(|e| e.to_string())?;
    let db = state.db.read().map_err(|e| e.to_string())?;
    pdb.get_preview_data(&db, &file_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn repack_portable_db(
    local_db_path: String,
    state: State<'_, AppState>,
) -> Result<PortableHeader, String> {
    let db = state.db.read().map_err(|e| e.to_string())?;
    let pdb_path = db
        .get_portable_meta("portable_db_path")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "portable db not initialized".to_string())?;
    drop(db);

    let mut pdb =
        cybermanju_portable_db::PortableDatabase::open(&pdb_path).map_err(|e| e.to_string())?;
    pdb.repack(&local_db_path, None)
        .map_err(|e| e.to_string())?;
    Ok(pdb.header().clone())
}

#[tauri::command]
pub fn mark_deletion_propagated(
    record_id: String,
    platform: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let db = state.db.write().map_err(|e| e.to_string())?;
    db.mark_deletion_propagated(&record_id, &platform)
        .map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub fn get_portable_db_meta(state: State<'_, AppState>) -> Result<Vec<(String, String)>, String> {
    let db = state.db.read().map_err(|e| e.to_string())?;
    db.get_all_portable_meta().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_recovery_entry(file_id: String, state: State<'_, AppState>) -> Result<bool, String> {
    let db = state.db.read().map_err(|e| e.to_string())?;
    let pdb_path = db
        .get_portable_meta("portable_db_path")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "portable db not initialized".to_string())?;
    drop(db);

    let pdb =
        cybermanju_portable_db::PortableDatabase::open(&pdb_path).map_err(|e| e.to_string())?;
    let db = state.db.write().map_err(|e| e.to_string())?;
    pdb.delete_recovery_entry(&db, &file_id)
        .map_err(|e| e.to_string())
}
