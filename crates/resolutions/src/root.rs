use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::shard::{ErasureParams, ShardType};

/// Magic bytes for `root.cybermanju` files.
pub const ROOT_MAGIC: &str = "CYBROOT__V2\0\0\0\0\0\0\0\0\0\0\0\0";

/// Root file format version.
pub const ROOT_VERSION: &str = "2.0";

/// Minimal plaintext header for `root.cybermanju`.
///
/// Total plaintext: ~128 bytes (magic + lengths + revocation root + signature).
/// Library name, file counts, shard distribution, encryption keys — all encrypted.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootHeader {
    /// Magic bytes for file identification (32 bytes).
    pub magic: String,
    /// Root file format version.
    pub version: String,
    /// Unique library identifier (e.g., "lib_001").
    pub library_id: String,
    /// Total length of the encrypted payload in bytes.
    pub encrypted_payload_len: u64,
    /// Length of the ML-DSA-65 signature in bytes.
    pub signature_len: u32,
}

impl Default for RootHeader {
    fn default() -> Self {
        Self {
            magic: ROOT_MAGIC.to_string(),
            version: ROOT_VERSION.to_string(),
            library_id: String::new(),
            encrypted_payload_len: 0,
            signature_len: 3200,
        }
    }
}

/// The encrypted payload of `root.cybermanju`.
///
/// Decrypted with the index_key. Contains all library metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootPayload {
    /// Human-readable library name.
    pub library_name: String,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
    /// ISO 8601 last modified timestamp.
    pub modified_at: String,
    /// Application version that created this payload.
    pub app_version: String,
    /// Total number of files in the library.
    pub total_files: u32,
    /// Total number of shards in the library.
    pub total_shards: u32,
    /// Total size of all files in bytes.
    pub total_size_bytes: u64,
    /// Total preview storage size in bytes.
    pub total_preview_size: u64,
    /// Total parity storage size in bytes.
    pub total_parity_size: u64,
    /// Distribution of shards across backends.
    pub shard_distribution: HashMap<String, u32>,
    /// Erasure codec name (e.g., "clay-codes").
    pub erasure_codec: String,
    /// Erasure coding parameters.
    pub erasure_params: ErasureParams,
    /// Encryption key information.
    pub encryption: EncryptionInfo,
    /// Sync configuration.
    pub sync: SyncInfo,
    /// BLAKE3 hash of the revocation Merkle tree root.
    pub revocation_merkle_root: String,
}

/// Encryption key information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionInfo {
    /// Encryption algorithm name.
    pub algorithm: String,
    /// Key ID for index encryption.
    pub index_key_id: String,
    /// Key ID for content encryption.
    pub content_key_id: String,
    /// Key ID for preview encryption.
    pub preview_key_id: String,
}

/// Sync configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncInfo {
    /// CRDT type (e.g., "delta-state").
    pub crdt: String,
    /// Whether vector clocks are enabled.
    pub vector_clock: bool,
    /// BLAKE3 hash of the last sync state.
    pub last_sync_hash: String,
}

/// Shard index within the root file. Contains metadata for every shard.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootShardIndex {
    /// Map of shard_id to shard info.
    pub shards: HashMap<String, RootShardInfo>,
    /// BLAKE3 Merkle root of all shard hashes.
    pub shard_merkle_root: String,
}

/// Shard info within the root file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootShardInfo {
    /// Type of shard.
    pub shard_type: ShardType,
    /// Backend storing this shard.
    pub backend: String,
    /// Remote path on the backend.
    pub remote_path: String,
    /// Remote URL for direct access (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_url: Option<String>,
    /// Number of files in this shard.
    pub file_count: u32,
    /// Total size of this shard in bytes.
    pub size_bytes: u64,
    /// BLAKE3 hash of the shard.
    pub blake3: String,
    /// ML-DSA-65 signature of the shard (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shard_signature: Option<String>,
    /// ISO 8601 timestamp of last verification (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_verified: Option<String>,
    /// Which shards this parity shard covers (optional).
    #[serde(default)]
    pub covers_shards: Vec<String>,
}

/// File manifest within the root file. Contains all file metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootFileManifest {
    /// Map of file_id to file entry.
    pub files: HashMap<String, RootFileEntry>,
    /// Folder structure.
    pub folders: HashMap<String, FolderEntry>,
    /// Tag → file_ids index.
    pub tags_index: HashMap<String, Vec<String>>,
    /// Face group → file_ids index.
    pub face_index: HashMap<String, Vec<String>>,
}

/// A file entry in the root manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootFileEntry {
    /// Original file name.
    pub name: String,
    /// MIME type.
    pub mime: String,
    /// Virtual folder path.
    pub folder: String,
    /// Original file size in bytes.
    pub original_size: u64,
    /// BLAKE3 hash of the original file.
    pub original_blake3: String,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
    /// File tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Face group IDs.
    #[serde(default)]
    pub face_groups: Vec<String>,
    /// Map of resolution level → shard IDs that contain this resolution.
    pub shard_assignments: HashMap<String, Vec<String>>,
    /// BLAKE3 Merkle root for this file's resolutions.
    pub merkle_root: String,
}

/// Folder entry in the root manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderEntry {
    /// File IDs directly in this folder.
    pub file_ids: Vec<String>,
    /// Subfolder names.
    pub subfolders: Vec<String>,
}

/// Resolution distribution policy across backends.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolutionDistribution {
    /// Distribution policy per resolution level (r0, r1, r2, r3, parity).
    pub resolution_distribution: HashMap<String, DistributionPolicy>,
    /// Target shard size in bytes.
    pub shard_size_target_bytes: u64,
    /// Strategy for splitting files into shards (e.g., "by_folder").
    pub shard_split_strategy: String,
    /// Cost model for storage backends.
    pub cost_model: CostModel,
}

/// Distribution policy for a single resolution level.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistributionPolicy {
    /// Which backends to use for this resolution.
    pub backends: Vec<String>,
    /// Redundancy level ("max", or a number).
    #[serde(rename = "type")]
    pub redundancy: serde_json::Value,
    /// Erasure coding config (false/none or detailed config).
    pub erasure: serde_json::Value,
    /// Priority level ("instant", "fast", "normal", "background").
    pub priority: String,
}

/// Erasure coding configuration within distribution policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErasureConfig {
    /// Erasure codec name.
    pub codec: String,
    /// Erasure coding parameters.
    pub params: ErasureParams,
}

/// Cost model for storage backends.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostModel {
    /// Cost per GB per month for each backend.
    pub storage_cost_per_gb_month: HashMap<String, f64>,
}

/// Complete `root.cybermanju` binary structure.
///
/// Binary layout:
/// ```text
/// Offset    Size      Field                           Encryption
/// ──────────────────────────────────────────────────────────────
/// [0..32)   32B       Magic: "CYBROOT__V2..."         PLAINTEXT
/// [32..36)  4B        header_len (u32 LE)             PLAINTEXT
/// [36..+h)  variable  header_json (RootHeader)        PLAINTEXT (minimal)
/// [h..+i)   4B        encrypted_shard_index_len      PLAINTEXT
/// [+i..+j)  variable  encrypted_shard_index_blob     ENCRYPTED (index_key)
/// [j..+k)   4B        encrypted_file_manifest_len    PLAINTEXT
/// [+k..+m)  variable  encrypted_file_manifest_blob   ENCRYPTED (index_key)
/// [m..+n)   4B        encrypted_distribution_len     PLAINTEXT
/// [+n..+p)  variable  encrypted_distribution_blob    ENCRYPTED (index_key)
/// [p..+q)   4B        revocation_merkle_root_len     PLAINTEXT
/// [q..+r)   32B       revocation_merkle_root         PLAINTEXT (for verification)
/// [r..+s)   4B        signature_len (u32 LE)         PLAINTEXT
/// [s..+t)   variable  root_signature                 PLAINTEXT (ML-DSA-65)
/// ```
#[derive(Debug, Clone)]
pub struct RootCybermanju {
    /// Plaintext header.
    pub header: RootHeader,
    /// Encrypted shard index blob (decrypted with index_key).
    pub encrypted_shard_index: Vec<u8>,
    /// Encrypted file manifest blob (decrypted with index_key).
    pub encrypted_file_manifest: Vec<u8>,
    /// Encrypted distribution policy blob (decrypted with index_key).
    pub encrypted_distribution: Vec<u8>,
    /// Revocation Merkle tree root (32 bytes, plaintext for verification).
    pub revocation_merkle_root: [u8; 32],
    /// ML-DSA-65 signature (plaintext for verification).
    pub root_signature: Vec<u8>,
}

/// Shard assignment per resolution level within a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShardAssignment {
    /// Resolution level (r0, r1, r2, r3).
    pub level: String,
    /// Shard IDs that contain this resolution.
    pub shards: Vec<String>,
}
