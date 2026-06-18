use std::path::PathBuf;
use tauri::State;

use crate::AppState;

#[tauri::command]
pub fn get_crash_log(data_dir: State<'_, AppState>) -> Result<Option<String>, String> {
    let path = PathBuf::from(&data_dir.data_dir).join("crash.log");
    if !path.exists() {
        return Ok(None);
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            if content.trim().is_empty() || content.contains("startup in progress") {
                let _ = std::fs::remove_file(&path);
                Ok(None)
            } else {
                Ok(Some(content))
            }
        }
        Err(e) => Err(format!("Failed to read crash log: {}", e)),
    }
}

#[tauri::command]
pub fn clear_crash_log(data_dir: State<'_, AppState>) -> Result<(), String> {
    let path = PathBuf::from(&data_dir.data_dir).join("crash.log");
    let _ = std::fs::remove_file(&path);
    Ok(())
}

#[tauri::command]
pub fn get_app_log(data_dir: State<'_, AppState>) -> Result<Option<String>, String> {
    let path = PathBuf::from(&data_dir.data_dir).join("cybermanju.log");
    if !path.exists() {
        return Ok(None);
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => Ok(Some(content)),
        Err(e) => Err(format!("Failed to read app log: {}", e)),
    }
}
