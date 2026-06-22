use crate::db::schema::AuditEntry;
use crate::AppState;
use redb::ReadableTable;
use serde::Serialize;
use tauri::State;

/// Fetch audit log entries with optional limit and entity filter.
#[tauri::command]
pub fn get_audit_log(
    limit: Option<u32>,
    entity_type: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<AuditEntry>, String> {
    let db = state.db.read().map_err(|e| e.to_string())?;
    let tx = db.begin_read().map_err(|e| e.to_string())?;
    let table = tx
        .open_table(crate::db::Database::get_audit_log_table())
        .map_err(|e| e.to_string())?;

    let mut entries: Vec<AuditEntry> = table
        .iter()
        .map_err(|e| e.to_string())?
        .filter_map(|entry| {
            let (_, value) = entry.ok()?;
            serde_json::from_str::<AuditEntry>(value.value()).ok()
        })
        .filter(|e| {
            if let Some(ref et) = entity_type {
                e.entity_type == *et
            } else {
                true
            }
        })
        .collect();

    // Sort by timestamp descending (most recent first)
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let limit = limit.unwrap_or(100) as usize;
    entries.truncate(limit);
    Ok(entries)
}

#[derive(Serialize)]
pub struct AuditChainResult {
    pub is_valid: bool,
    pub broken_at: Option<u32>,
    pub entry_count: u32,
}

/// Verify the tamper-evident chain of audit log entries.
#[tauri::command]
pub fn verify_audit_chain(state: State<'_, AppState>) -> Result<AuditChainResult, String> {
    let db = state.db.read().map_err(|e| e.to_string())?;
    let tx = db.begin_read().map_err(|e| e.to_string())?;
    let table = tx
        .open_table(crate::db::Database::get_audit_log_table())
        .map_err(|e| e.to_string())?;

    let mut entries: Vec<AuditEntry> = table
        .iter()
        .map_err(|e| e.to_string())?
        .filter_map(|entry| {
            let (_, value) = entry.ok()?;
            serde_json::from_str::<AuditEntry>(value.value()).ok()
        })
        .collect();

    // Sort by timestamp ascending (oldest first) for chain verification
    entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    let entry_count = entries.len() as u32;
    let mut prev_hash = String::new();

    for (i, entry) in entries.iter().enumerate() {
        // Verify prev_hash links correctly
        if entry.prev_hash != prev_hash {
            return Ok(AuditChainResult {
                is_valid: false,
                broken_at: Some(i as u32),
                entry_count,
            });
        }

        // Recompute entry_hash
        let mut entry_for_hash = entry.clone();
        entry_for_hash.entry_hash = String::new();
        let json = serde_json::to_string(&entry_for_hash).unwrap_or_default();
        let computed_hash = blake3::hash(json.as_bytes()).to_hex().to_string();

        if computed_hash != entry.entry_hash {
            return Ok(AuditChainResult {
                is_valid: false,
                broken_at: Some(i as u32),
                entry_count,
            });
        }

        prev_hash = entry.entry_hash.clone();
    }

    Ok(AuditChainResult {
        is_valid: true,
        broken_at: None,
        entry_count,
    })
}
