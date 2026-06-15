use std::collections::HashMap;
use tauri::State;

use crate::db::schema::FileNode;
use crate::AppState;

/// Find all duplicate files grouped by their BLAKE3 hash.
/// Returns groups of files that share the same hash (i.e., identical content).
#[tauri::command]
pub fn find_duplicates(state: State<'_, AppState>) -> Result<Vec<Vec<FileNode>>, String> {
    let db = state.db.read().map_err(|e| e.to_string())?;
    let tx = db.begin_read().map_err(|e| e.to_string())?;
    let table = tx
        .open_table(crate::db::Database::get_files_table())
        .map_err(|e| e.to_string())?;

    let mut by_hash: HashMap<Option<String>, Vec<FileNode>> = HashMap::new();

    for entry in table.iter().map_err(|e| e.to_string())? {
        let (_, value) = entry.map_err(|e| e.to_string())?;
        if let Ok(node) = serde_json::from_str::<FileNode>(value.value()) {
            if node.hash_blake3.is_some() {
                by_hash
                    .entry(node.hash_blake3.clone())
                    .or_default()
                    .push(node);
            }
        }
    }

    let duplicates: Vec<Vec<FileNode>> = by_hash
        .into_values()
        .filter(|group| group.len() > 1)
        .collect();

    Ok(duplicates)
}
