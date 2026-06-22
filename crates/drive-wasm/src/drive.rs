use crate::types::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = cybermanjuStorage)]
    fn persist_file_node(json: &str) -> JsValue;

    #[wasm_bindgen(js_namespace = cybermanjuStorage)]
    fn load_all_files() -> JsValue;

    #[wasm_bindgen(js_namespace = cybermanjuStorage)]
    fn remove_file_node(id: &str) -> JsValue;
}

fn persist_node(node: &WasmFileNode) {
    if let Ok(json) = serde_json::to_string(node) {
        let _ = persist_file_node(&json);
    }
}

fn remove_node(id: &str) {
    let _ = remove_file_node(id);
}

#[wasm_bindgen]
pub struct VirtualDrive {
    files: Vec<WasmFileNode>,
    quota: DriveQuota,
}

#[wasm_bindgen]
impl VirtualDrive {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            quota: DriveQuota {
                used_bytes: 0,
                total_bytes: 0,
                file_count: 0,
                folder_count: 0,
            },
        }
    }

    /// Hydrate from IndexedDB on startup.
    pub fn hydrate_from_storage() -> Self {
        let mut drive = VirtualDrive::new();
        let js_val = load_all_files();
        if !js_val.is_undefined() && !js_val.is_null() {
            if let Ok(arr) = serde_wasm_bindgen::from_value::<Vec<WasmFileNode>>(js_val) {
                for f in arr {
                    drive.update_quota(&f);
                    drive.files.push(f);
                }
            }
        }
        drive
    }

    pub fn create_file(
        &mut self,
        name: &str,
        file_type: &str,
        parent_id: Option<String>,
    ) -> Result<JsValue, JsValue> {
        let node = WasmFileNode::new(name.to_string(), file_type.to_string(), parent_id);
        let size = node.size_bytes;
        let is_folder = node.file_type == "folder";
        let id = node.id.clone();
        persist_node(&node);
        self.files.push(node);
        self.quota.used_bytes += size;
        if is_folder {
            self.quota.folder_count += 1;
        } else {
            self.quota.file_count += 1;
        }
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &JsValue::from_str("id"), &JsValue::from_str(&id))?;
        Ok(obj.into())
    }

    pub fn delete_file(&mut self, file_id: &str) -> Result<(), JsValue> {
        let idx = self
            .files
            .iter()
            .position(|f| f.id == file_id)
            .ok_or_else(|| JsValue::from_str("File not found"))?;
        let removed = self.files.remove(idx);
        remove_node(file_id);
        if removed.file_type == "folder" {
            self.quota.folder_count = self.quota.folder_count.saturating_sub(1);
        } else {
            self.quota.file_count = self.quota.file_count.saturating_sub(1);
        }
        self.quota.used_bytes = self.quota.used_bytes.saturating_sub(removed.size_bytes);
        Ok(())
    }

    pub fn get_file(&self, file_id: &str) -> Result<JsValue, JsValue> {
        let file = self
            .files
            .iter()
            .find(|f| f.id == file_id)
            .ok_or_else(|| JsValue::from_str("File not found"))?;
        serde_wasm_bindgen::to_value(file)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    pub fn list_files(&self, parent_id: Option<String>) -> Result<JsValue, JsValue> {
        let filtered: Vec<&WasmFileNode> = match parent_id {
            Some(pid) => self
                .files
                .iter()
                .filter(|f| f.parent_id.as_deref() == Some(&pid))
                .collect(),
            None => self.files.iter().collect(),
        };
        let mut result = Vec::with_capacity(filtered.len());
        for f in filtered {
            result.push(
                serde_wasm_bindgen::to_value(f)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?,
            );
        }
        Ok(js_sys::Array::from_iter(result).into())
    }

    pub fn search_files(&self, query: &str) -> Result<JsValue, JsValue> {
        let q = query.to_lowercase();
        let filtered: Vec<&WasmFileNode> = self
            .files
            .iter()
            .filter(|f| {
                f.name.to_lowercase().contains(&q)
                    || f.tags.iter().any(|t| t.to_lowercase().contains(&q))
            })
            .collect();
        let mut result = Vec::with_capacity(filtered.len());
        for f in filtered {
            result.push(
                serde_wasm_bindgen::to_value(f)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?,
            );
        }
        Ok(js_sys::Array::from_iter(result).into())
    }

    pub fn rename_file(&mut self, file_id: &str, new_name: &str) -> Result<(), JsValue> {
        let file = self
            .files
            .iter_mut()
            .find(|f| f.id == file_id)
            .ok_or_else(|| JsValue::from_str("File not found"))?;
        file.name = new_name.to_string();
        file.modified_at = chrono::Utc::now().to_rfc3339();
        persist_node(file);
        Ok(())
    }

    pub fn move_file(
        &mut self,
        file_id: &str,
        new_parent_id: Option<String>,
    ) -> Result<(), JsValue> {
        let file = self
            .files
            .iter_mut()
            .find(|f| f.id == file_id)
            .ok_or_else(|| JsValue::from_str("File not found"))?;
        file.parent_id = new_parent_id;
        file.modified_at = chrono::Utc::now().to_rfc3339();
        persist_node(file);
        Ok(())
    }

    pub fn set_file_size(&mut self, file_id: &str, size_bytes: u64) -> Result<(), JsValue> {
        let file = self
            .files
            .iter_mut()
            .find(|f| f.id == file_id)
            .ok_or_else(|| JsValue::from_str("File not found"))?;
        let diff = if size_bytes > file.size_bytes {
            size_bytes - file.size_bytes
        } else {
            file.size_bytes - size_bytes
        };
        file.size_bytes = size_bytes;
        self.quota.used_bytes = if size_bytes > file.size_bytes {
            self.quota.used_bytes + diff
        } else {
            self.quota.used_bytes.saturating_sub(diff)
        };
        file.modified_at = chrono::Utc::now().to_rfc3339();
        persist_node(file);
        Ok(())
    }

    pub fn set_file_tags(&mut self, file_id: &str, tags: Vec<String>) -> Result<(), JsValue> {
        let file = self
            .files
            .iter_mut()
            .find(|f| f.id == file_id)
            .ok_or_else(|| JsValue::from_str("File not found"))?;
        file.tags = tags;
        file.modified_at = chrono::Utc::now().to_rfc3339();
        persist_node(file);
        Ok(())
    }

    pub fn toggle_star(&mut self, file_id: &str) -> Result<bool, JsValue> {
        let file = self
            .files
            .iter_mut()
            .find(|f| f.id == file_id)
            .ok_or_else(|| JsValue::from_str("File not found"))?;
        file.is_starred = !file.is_starred;
        file.modified_at = chrono::Utc::now().to_rfc3339();
        persist_node(file);
        Ok(file.is_starred)
    }

    pub fn get_starred_files(&self) -> Result<JsValue, JsValue> {
        let filtered: Vec<&WasmFileNode> = self.files.iter().filter(|f| f.is_starred).collect();
        let mut result = Vec::with_capacity(filtered.len());
        for f in filtered {
            result.push(
                serde_wasm_bindgen::to_value(f)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?,
            );
        }
        Ok(js_sys::Array::from_iter(result).into())
    }

    pub fn get_geo_files(&self) -> Result<JsValue, JsValue> {
        let filtered: Vec<&WasmFileNode> = self
            .files
            .iter()
            .filter(|f| f.gps_lat.is_some() && f.gps_lon.is_some())
            .collect();
        let mut result = Vec::with_capacity(filtered.len());
        for f in filtered {
            result.push(
                serde_wasm_bindgen::to_value(f)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?,
            );
        }
        Ok(js_sys::Array::from_iter(result).into())
    }

    pub fn get_quota(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.quota)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    pub fn get_all_files(&self) -> Result<JsValue, JsValue> {
        let mut result = Vec::with_capacity(self.files.len());
        for f in &self.files {
            result.push(
                serde_wasm_bindgen::to_value(f)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?,
            );
        }
        Ok(js_sys::Array::from_iter(result).into())
    }

    pub fn file_count(&self) -> u32 {
        self.quota.file_count
    }
    pub fn folder_count(&self) -> u32 {
        self.quota.folder_count
    }
    pub fn total_bytes(&self) -> u64 {
        self.quota.used_bytes
    }

    pub fn to_json(&self) -> Result<String, JsValue> {
        let data = serde_json::to_string(&self.files)
            .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))?;
        Ok(data)
    }

    pub fn from_json(json: &str) -> Result<VirtualDrive, JsValue> {
        let files: Vec<WasmFileNode> = serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("JSON deserialization error: {}", e)))?;
        let mut drive = VirtualDrive::new();
        for f in files {
            drive.update_quota(&f);
            drive.files.push(f);
        }
        Ok(drive)
    }

    fn update_quota(&mut self, file: &WasmFileNode) {
        self.quota.used_bytes += file.size_bytes;
        if file.file_type == "folder" {
            self.quota.folder_count += 1;
        } else {
            self.quota.file_count += 1;
        }
    }
}
