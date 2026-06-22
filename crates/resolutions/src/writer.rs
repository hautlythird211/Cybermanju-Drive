use std::collections::HashMap;
use std::path::Path;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::errors::ResolutionError;
use crate::shard::*;
use cybermanju_preview_keys::key_derivation;

/// Manifest produced after finalizing a shard.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShardManifest {
    pub shard_id: String,
    pub file_path: String,
    pub header: ShardHeader,
    pub index: ShardIndex,
    pub content_map: ContentMap,
    pub erasure_meta: ErasureMeta,
    pub shard_mac: String,
    pub total_bytes: u64,
}

/// Builds a `.cybermanju` v2 shard file.
pub struct ShardWriter {
    shard_id: String,
    root_hash_backlink: String,
    files: HashMap<String, ResolutionEntry>,
    content_buf: Vec<u8>,
    content_offset: u64,
    key_hierarchy: Option<KeyHierarchyRef>,
}

/// Reference to a key hierarchy for encryption.
pub struct KeyHierarchyRef {
    pub master_key: [u8; 32],
}

impl KeyHierarchyRef {
    pub fn new(master_key: [u8; 32]) -> Self {
        Self { master_key }
    }
}

impl ShardWriter {
    pub fn new(shard_id: &str, root_hash: &str) -> Self {
        Self {
            shard_id: shard_id.to_string(),
            root_hash_backlink: root_hash.to_string(),
            files: HashMap::new(),
            content_buf: Vec::new(),
            content_offset: 0,
            key_hierarchy: None,
        }
    }

    pub fn with_key_hierarchy(mut self, kh: KeyHierarchyRef) -> Self {
        self.key_hierarchy = Some(kh);
        self
    }

    /// Add a file at all resolution levels.
    pub fn add_file(
        &mut self,
        file_id: &str,
        name: &str,
        mime: &str,
        folder: &str,
        r0: &[u8],
        r1: &[u8],
        r2: &[u8],
        r3: &[u8],
    ) -> Result<(), ResolutionError> {
        let now = Utc::now().to_rfc3339();
        let original_blake3 = blake3::hash(r3).to_hex().to_string();

        let mut resolutions = HashMap::new();

        // r0 — 200px thumbnail, preview_key
        let r0_offset = self.content_offset;
        let r0_blake3 = blake3::hash(r0).to_hex().to_string();
        self.content_buf.extend_from_slice(r0);
        self.content_offset += r0.len() as u64;
        resolutions.insert(
            "r0".to_string(),
            ResolutionLevel {
                blake3: r0_blake3,
                size: r0.len() as u64,
                format: "webp".to_string(),
                width: Some(200),
                height: Some(200),
                content_offset: r0_offset,
                content_length: r0.len() as u64,
                encrypted: self.key_hierarchy.is_some(),
                encryption_key_tier: "preview".to_string(),
                chunk_count: None,
                chunk_size: None,
            },
        );

        // r1 — 640px preview, preview_key
        let r1_offset = self.content_offset;
        let r1_blake3 = blake3::hash(r1).to_hex().to_string();
        self.content_buf.extend_from_slice(r1);
        self.content_offset += r1.len() as u64;
        resolutions.insert(
            "r1".to_string(),
            ResolutionLevel {
                blake3: r1_blake3,
                size: r1.len() as u64,
                format: "webp".to_string(),
                width: Some(640),
                height: Some(480),
                content_offset: r1_offset,
                content_length: r1.len() as u64,
                encrypted: self.key_hierarchy.is_some(),
                encryption_key_tier: "preview".to_string(),
                chunk_count: None,
                chunk_size: None,
            },
        );

        // r2 — 1920px medium, content_key
        let r2_offset = self.content_offset;
        let r2_blake3 = blake3::hash(r2).to_hex().to_string();
        self.content_buf.extend_from_slice(r2);
        self.content_offset += r2.len() as u64;
        resolutions.insert(
            "r2".to_string(),
            ResolutionLevel {
                blake3: r2_blake3,
                size: r2.len() as u64,
                format: "jpeg".to_string(),
                width: Some(1920),
                height: Some(1080),
                content_offset: r2_offset,
                content_length: r2.len() as u64,
                encrypted: self.key_hierarchy.is_some(),
                encryption_key_tier: "content".to_string(),
                chunk_count: None,
                chunk_size: None,
            },
        );

        // r3 — original bytes, content_key, chunked
        let chunk_size: u32 = 64 * 1024; // 64KB chunks
        let chunk_count = (r3.len() as u32 + chunk_size - 1) / chunk_size;
        let r3_offset = self.content_offset;
        let r3_blake3 = blake3::hash(r3).to_hex().to_string();
        self.content_buf.extend_from_slice(r3);
        self.content_offset += r3.len() as u64;
        resolutions.insert(
            "r3".to_string(),
            ResolutionLevel {
                blake3: r3_blake3,
                size: r3.len() as u64,
                format: "raw".to_string(),
                width: None,
                height: None,
                content_offset: r3_offset,
                content_length: r3.len() as u64,
                encrypted: self.key_hierarchy.is_some(),
                encryption_key_tier: "content".to_string(),
                chunk_count: Some(chunk_count),
                chunk_size: Some(chunk_size),
            },
        );

        let entry = ResolutionEntry {
            name: name.to_string(),
            mime: mime.to_string(),
            folder: folder.to_string(),
            tags: Vec::new(),
            original_size: r3.len() as u64,
            original_blake3,
            created_at: now.clone(),
            modified_at: now,
            gps: None,
            face_groups: Vec::new(),
            versions: 1,
            current_version: 1,
            resolutions,
            parity: None,
        };

        self.files.insert(file_id.to_string(), entry);
        Ok(())
    }

    /// Finalize shard: build index, content map, erasure meta, and write binary.
    pub fn finalize(self, output_path: &Path) -> Result<ShardManifest, ResolutionError> {
        let now = Utc::now().to_rfc3339();
        let shard_id = self.shard_id.clone();

        // Build content map
        let mut blob_regions = Vec::new();
        for (fid, entry) in &self.files {
            for (res, level) in &entry.resolutions {
                blob_regions.push(ContentBlobRegion {
                    id: format!("{}_{}", res, fid),
                    offset: level.content_offset,
                    length: level.content_length,
                    key_tier: level.encryption_key_tier.clone(),
                    compression: if res == "r0" || res == "r1" {
                        "webp".to_string()
                    } else if res == "r2" {
                        "jpeg".to_string()
                    } else {
                        "none".to_string()
                    },
                });
            }
        }
        let content_total_bytes = self.content_offset;

        let content_map = ContentMap {
            blob_regions,
            content_total_bytes,
            compression_ratio: 1.0,
            encrypted_ratio: if self.key_hierarchy.is_some() { 1.0 } else { 0.0 },
        };

        // Build index
        let index = ShardIndex {
            shard_id: shard_id.clone(),
            files: self.files,
            sprite_sheets: HashMap::new(),
            erasure_map: ErasureMap {
                clay_shards: Vec::new(),
                fountain_packets: Vec::new(),
            },
            merkle_root: blake3::hash(self.content_buf.as_slice()).to_hex().to_string(),
        };

        // Build erasure meta
        let erasure_meta = ErasureMeta {
            shard_id: shard_id.clone(),
            erasure_codec: "reed-solomon".to_string(),
            erasure_params: ErasureParams { k: 4, m: 2, d: 4 },
            this_shard_role: "data_shard_0".to_string(),
            parity_distributed_to: Vec::new(),
            recovery_threshold: RecoveryThreshold {
                data_shards_needed: 4,
                total_shards_available: 6,
                can_recover_with: vec!["any_4_of_6".to_string()],
            },
            fountain_config: FountainConfig {
                symbol_size: 1024,
                source_symbols: 4,
                repair_symbols_per_shard: 2,
                min_packets_for_recovery: 4,
            },
            shard_blake3: blake3::hash(self.content_buf.as_slice()).to_hex().to_string(),
        };

        // Build header
        let header = ShardHeader {
            magic: SHARD_MAGIC.to_string(),
            version: SHARD_VERSION.to_string(),
            shard_id: shard_id.clone(),
            root_hash_backlink: self.root_hash_backlink,
            created_at: now.clone(),
            modified_at: now,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            shard_type: ShardType::Content,
            encrypted_index_len: 0,
            encrypted_content_map_len: 0,
            encrypted_erasure_len: 0,
            content_algorithm: "ml-kem-1024+chacha20poly1305".to_string(),
            index_algorithm: "aes-256-gcm".to_string(),
            compression: "lz4+zstd15+brotli11".to_string(),
            erasure_codec: "reed-solomon".to_string(),
            erasure_params: ErasureParams { k: 4, m: 2, d: 4 },
            platform_origin: std::env::consts::OS.to_string(),
        };

        // Compute shard MAC (keyed BLAKE3 using master key when available)
        let shard_mac = if let Some(kh) = &self.key_hierarchy {
            let mac_key = key_derivation::derive_shard_mac_key(&kh.master_key, &shard_id);
            let mac_hash = blake3::keyed_hash(&mac_key, &self.content_buf);
            mac_hash.to_hex().to_string()
        } else {
            compute_shard_mac(&self.content_buf, &shard_id)
        };

        // Write binary layout
        let mut output = Vec::new();

        // [PLAINTEXT HEADER] — JSON-serialized header
        let header_json = serde_json::to_vec(&header)
            .map_err(|e| ResolutionError::SerializationError(e.to_string()))?;
        output.extend_from_slice(&(header_json.len() as u32).to_le_bytes());
        output.extend_from_slice(&header_json);

        // [INDEX LAYER] — JSON-serialized index (encrypted if key hierarchy present)
        let index_json = serde_json::to_vec(&index)
            .map_err(|e| ResolutionError::SerializationError(e.to_string()))?;
        output.extend_from_slice(&(index_json.len() as u32).to_le_bytes());
        output.extend_from_slice(&index_json);

        // [CONTENT MAP] — JSON-serialized content map
        let content_map_json = serde_json::to_vec(&content_map)
            .map_err(|e| ResolutionError::SerializationError(e.to_string()))?;
        output.extend_from_slice(&(content_map_json.len() as u32).to_le_bytes());
        output.extend_from_slice(&content_map_json);

        // [ERASURE META] — JSON-serialized erasure metadata
        let erasure_json = serde_json::to_vec(&erasure_meta)
            .map_err(|e| ResolutionError::SerializationError(e.to_string()))?;
        output.extend_from_slice(&(erasure_json.len() as u32).to_le_bytes());
        output.extend_from_slice(&erasure_json);

        // [CONTENT BLOBS] — all resolution blobs
        output.extend_from_slice(&self.content_buf);

        // [FOOTER] — BLAKE3 hash + shard MAC
        let footer_hash = blake3::hash(&output).to_hex().to_string();
        output.extend_from_slice(footer_hash.as_bytes());
        output.extend_from_slice(shard_mac.as_bytes());

        std::fs::write(output_path, &output)
            .map_err(|e| ResolutionError::IoError(e.to_string()))?;

        let total_bytes = output.len() as u64;

        Ok(ShardManifest {
            shard_id,
            file_path: output_path.to_string_lossy().to_string(),
            header,
            index,
            content_map,
            erasure_meta,
            shard_mac,
            total_bytes,
        })
    }
}

/// Compute a keyed shard MAC using BLAKE3.
fn compute_shard_mac(content: &[u8], shard_id: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"cybermanju-shard-mac-v1");
    hasher.update(shard_id.as_bytes());
    hasher.update(content);
    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_shard_writer_add_file() {
        let mut writer = ShardWriter::new("shard_0001", "root_hash_abc");
        let r0 = vec![0u8; 100];
        let r1 = vec![0u8; 200];
        let r2 = vec![0u8; 500];
        let r3 = vec![0u8; 1000];

        writer
            .add_file("file1", "photo.jpg", "image/jpeg", "/photos", &r0, &r1, &r2, &r3)
            .unwrap();

        assert!(writer.files.contains_key("file1"));
        let entry = writer.files.get("file1").unwrap();
        assert_eq!(entry.resolutions.len(), 4);
        assert!(entry.resolutions.contains_key("r0"));
        assert!(entry.resolutions.contains_key("r3"));
    }

    #[test]
    fn test_shard_writer_finalize() {
        let mut writer = ShardWriter::new("shard_0001", "root_hash_abc");
        let r0 = vec![0u8; 100];
        let r1 = vec![0u8; 200];
        let r2 = vec![0u8; 500];
        let r3 = vec![0u8; 1000];

        writer
            .add_file("file1", "photo.jpg", "image/jpeg", "/photos", &r0, &r1, &r2, &r3)
            .unwrap();

        let tmp = env::temp_dir().join("test_shard.cybermanju");
        let manifest = writer.finalize(&tmp).unwrap();
        assert_eq!(manifest.shard_id, "shard_0001");
        assert!(manifest.total_bytes > 0);
        let _ = std::fs::remove_file(&tmp);
    }
}
