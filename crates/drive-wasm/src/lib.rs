pub mod crypto;
pub mod compression;
pub mod types;
pub mod sync;
pub mod drive;

use wasm_bindgen::prelude::*;

pub use crypto::*;
pub use compression::*;
pub use sync::*;
pub use drive::*;

#[wasm_bindgen(start)]
pub fn init() {
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("Cybermanju Drive WASM module initialized");
}

/// Generate a v4 UUID
#[wasm_bindgen]
pub fn generate_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Get current UTC timestamp as RFC 3339 string
#[wasm_bindgen]
pub fn now_utc() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Compute BLAKE3 hash of data and return hex string
#[wasm_bindgen]
pub fn hash_file_meta(name: &str, size: u64, modified: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(name.as_bytes());
    hasher.update(&size.to_le_bytes());
    hasher.update(modified.as_bytes());
    hasher.finalize().to_hex().to_string()
}
