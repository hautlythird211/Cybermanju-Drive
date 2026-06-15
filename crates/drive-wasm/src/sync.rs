use crate::types::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SyncEngine {
    state: SyncState,
}

#[wasm_bindgen]
impl SyncEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            state: SyncState::new(),
        }
    }

    pub fn add_file(
        &mut self,
        path: &str,
        size_bytes: u64,
        backend: &str,
    ) -> Result<JsValue, JsValue> {
        let backend_type = parse_backend_type(backend)?;
        let entry = SyncFileEntry::new(path.to_string(), size_bytes, backend_type);
        self.state.entries.push(entry.clone());
        self.state.total_files += 1;
        self.state.total_bytes += size_bytes;
        serde_wasm_bindgen::to_value(&entry)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    pub fn mark_synced(&mut self, file_id: &str) -> Result<(), JsValue> {
        let entry = self
            .state
            .entries
            .iter_mut()
            .find(|e| e.id == file_id)
            .ok_or_else(|| JsValue::from_str("File not found in sync state"))?;
        entry.status = SyncStatus::Done;
        entry.synced_at = Some(chrono::Utc::now().to_rfc3339());
        entry.local_changes = 0;
        Ok(())
    }

    pub fn mark_error(&mut self, file_id: &str, error: &str) -> Result<(), JsValue> {
        let entry = self
            .state
            .entries
            .iter_mut()
            .find(|e| e.id == file_id)
            .ok_or_else(|| JsValue::from_str("File not found in sync state"))?;
        entry.status = SyncStatus::Error;
        entry.error_message = Some(error.to_string());
        Ok(())
    }

    pub fn mark_changed(&mut self, file_id: &str) -> Result<(), JsValue> {
        let entry = self
            .state
            .entries
            .iter_mut()
            .find(|e| e.id == file_id)
            .ok_or_else(|| JsValue::from_str("File not found in sync state"))?;
        entry.local_changes += 1;
        entry.status = SyncStatus::Scanning;
        Ok(())
    }

    pub fn remove_file(&mut self, file_id: &str) -> Result<(), JsValue> {
        let idx = self
            .state
            .entries
            .iter()
            .position(|e| e.id == file_id)
            .ok_or_else(|| JsValue::from_str("File not found in sync state"))?;
        let removed = self.state.entries.remove(idx);
        self.state.total_files = self.state.total_files.saturating_sub(1);
        self.state.total_bytes = self.state.total_bytes.saturating_sub(removed.size_bytes);
        Ok(())
    }

    pub fn get_entries(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.state.entries)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    pub fn get_state(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.state)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    pub fn get_pending_count(&self) -> u32 {
        self.state
            .entries
            .iter()
            .filter(|e| matches!(e.status, SyncStatus::Idle | SyncStatus::Scanning))
            .count() as u32
    }

    pub fn get_error_count(&self) -> u32 {
        self.state
            .entries
            .iter()
            .filter(|e| matches!(e.status, SyncStatus::Error))
            .count() as u32
    }

    pub fn get_synced_count(&self) -> u32 {
        self.state
            .entries
            .iter()
            .filter(|e| matches!(e.status, SyncStatus::Done | SyncStatus::Completed))
            .count() as u32
    }

    pub fn has_changes(&self) -> bool {
        self.state.entries.iter().any(|e| e.local_changes > 0)
    }

    pub fn reset(&mut self) {
        self.state = SyncState::new();
    }

    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.state)
            .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))
    }

    pub fn from_json(json: &str) -> Result<SyncEngine, JsValue> {
        let state: SyncState = serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("JSON deserialization error: {}", e)))?;
        Ok(Self { state })
    }
}

fn parse_backend_type(s: &str) -> Result<SyncBackendType, JsValue> {
    match s.to_lowercase().as_str() {
        "local" => Ok(SyncBackendType::Local),
        "github" => Ok(SyncBackendType::GitHub),
        "gitlab" => Ok(SyncBackendType::GitLab),
        "googledrive" | "google_drive" => Ok(SyncBackendType::GoogleDrive),
        "googlephotos" | "google_photos" => Ok(SyncBackendType::GooglePhotos),
        "telegram" => Ok(SyncBackendType::Telegram),
        _ => Err(JsValue::from_str(&format!("Unknown backend type: {}", s))),
    }
}
