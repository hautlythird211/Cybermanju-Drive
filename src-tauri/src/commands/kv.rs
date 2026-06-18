use crate::AppState;
use tauri::State;

/// Set a key-value pair in the persistent kv store.
#[tauri::command]
pub fn kv_set(key: String, value: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.read().map_err(|e| e.to_string())?;
    let tx = db.begin_write().map_err(|e| e.to_string())?;
    {
        let mut table = tx
            .open_table(crate::db::Database::get_kv_store_table())
            .map_err(|e| e.to_string())?;
        table
            .insert(key.as_str(), value.as_str())
            .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

/// Get a value by key from the persistent kv store.
#[tauri::command]
pub fn kv_get(key: String, state: State<'_, AppState>) -> Result<Option<String>, String> {
    let db = state.db.read().map_err(|e| e.to_string())?;
    let tx = db.begin_read().map_err(|e| e.to_string())?;
    let result = {
        let table = tx
            .open_table(crate::db::Database::get_kv_store_table())
            .map_err(|e| e.to_string())?;
        table
            .get(key.as_str())
            .map_err(|e| e.to_string())?
            .map(|v| v.value().to_string())
    };
    Ok(result)
}
