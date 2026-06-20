use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Magic bytes for `.cybermanju` shard files.
pub const SHARD_MAGIC: &str = "CYBSHARD_V2\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";

/// Shard format version.
pub const SHARD_VERSION: &str = "2.0";

/// Minimal plaintext header for a `.cybermanju` shard file.
///
/// Total plaintext: ~88 bytes. No file names, no sizes, no metadata.
/// An attacker cannot determine what's inside from the header alone.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShardHeader {
    /// Magic bytes for file identification (32 bytes).
    pub magic: String,
    /// Shard format version.
    pub version: String,
    /// Unique shard identifier (e.g., "shard_0042").
    pub shard_id: String,
    /// BLAKE3 hash of the root.cybermanju file this shard belongs to.
    pub root_hash_backlink: String,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
    /// ISO 8601 last modified timestamp.
    pub modified_at: String,
    /// Application version that created this shard.
    pub app_version: String,
    /// Shard type: "content", "parity", or "preview".
    pub shard_type: ShardType,
    /// Length of the encrypted index blob in bytes.
    pub encrypted_index_len: u32,
    /// Length of the encrypted content map blob in bytes.
    pub encrypted_content_map_len: u32,
    /// Length of the encrypted erasure metadata blob in bytes.
    pub encrypted_erasure_len: u32,
    /// Algorithm used for content encryption (e.g., "ml-kem-1024+chacha20poly1305").
    pub content_algorithm: String,
    /// Algorithm used for index encryption (e.g., "aes-256-gcm").
    pub index_algorithm: String,
    /// Compression algorithms applied (e.g., "lz4+zstd15+brotli11").
    pub compression: String,
    /// Erasure codec name (e.g., "clay-codes").
    pub erasure_codec: String,
    /// Erasure coding parameters (e.g., {"k": 3, "m": 1, "d": 4}).
    pub erasure_params: ErasureParams,
    /// Platform that created this shard (e.g., "linux", "windows", "macos").
    pub platform_origin: String,
}

impl Default for ShardHeader {
    fn default() -> Self {
        Self {
            magic: SHARD_MAGIC.to_string(),
            version: SHARD_VERSION.to_string(),
            shard_id: String::new(),
            root_hash_backlink: String::new(),
            created_at: String::new(),
            modified_at: String::new(),
            app_version: "0.1.0".to_string(),
            shard_type: ShardType::Content,
            encrypted_index_len: 0,
            encrypted_content_map_len: 0,
            encrypted_erasure_len: 0,
            content_algorithm: "ml-kem-1024+chacha20poly1305".to_string(),
            index_algorithm: "aes-256-gcm".to_string(),
            compression: "lz4+zstd15+brotli11".to_string(),
            erasure_codec: "clay-codes".to_string(),
            erasure_params: ErasureParams::default(),
            platform_origin: std::env::consts::OS.to_string(),
        }
    }
}

/// The type of shard.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShardType {
    /// Standard content shard containing file data.
    Content,
    /// Parity shard for erasure coding recovery.
    Parity,
    /// Preview-only shard with thumbnails and previews.
    Preview,
}

/// Erasure coding parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErasureParams {
    /// Number of data shards required for reconstruction.
    pub k: u32,
    /// Number of parity shards.
    pub m: u32,
    /// Distance parameter for erasure coding.
    pub d: u32,
}

impl Default for ErasureParams {
    fn default() -> Self {
        Self { k: 3, m: 1, d: 4 }
    }
}

/// Encrypted index within a shard. Contains the file manifest, blob map,
/// and resolution map. Encrypted with the index_key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShardIndex {
    /// The shard this index belongs to.
    pub shard_id: String,
    /// Map of file_id to resolution entries for all files in this shard.
    pub files: HashMap<String, ResolutionEntry>,
    /// Sprite sheet entries for batch thumbnails.
    pub sprite_sheets: HashMap<String, SpriteSheetEntry>,
    /// Erasure coding map showing clay shards and fountain packets.
    pub erasure_map: ErasureMap,
    /// BLAKE3 Merkle root of this shard's index contents.
    pub merkle_root: String,
}

/// A single file's resolution data within a shard.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolutionEntry {
    /// Original file name.
    pub name: String,
    /// MIME type of the original file.
    pub mime: String,
    /// Virtual folder path within the shard.
    pub folder: String,
    /// File tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Original file size in bytes.
    pub original_size: u64,
    /// BLAKE3 hash of the original file.
    pub original_blake3: String,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
    /// ISO 8601 modification timestamp.
    pub modified_at: String,
    /// GPS coordinates if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gps: Option<GpsCoords>,
    /// Face group IDs associated with this file.
    #[serde(default)]
    pub face_groups: Vec<String>,
    /// Total version count.
    #[serde(default = "default_version_count")]
    pub versions: u32,
    /// Current version number.
    #[serde(default = "default_version_count")]
    pub current_version: u32,
    /// Resolution levels available for this file (r0, r1, r2, r3).
    pub resolutions: HashMap<String, ResolutionLevel>,
    /// Parity coding information for this file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parity: Option<FileParity>,
}

fn default_version_count() -> u32 {
    1
}

/// GPS coordinates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GpsCoords {
    pub lat: f64,
    pub lon: f64,
}

/// A single resolution level for a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolutionLevel {
    /// BLAKE3 hash of this resolution's data.
    pub blake3: String,
    /// Size of this resolution in bytes.
    pub size: u64,
    /// Format of this resolution (e.g., "webp", "jpeg", "encrypted").
    pub format: String,
    /// Width in pixels (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    /// Height in pixels (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// Byte offset within the content section.
    pub content_offset: u64,
    /// Byte length within the content section.
    pub content_length: u64,
    /// Whether this resolution is encrypted.
    pub encrypted: bool,
    /// Key tier used for encryption ("preview" or "content").
    pub encryption_key_tier: String,
    /// Number of chunks for r3 files (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_count: Option<u32>,
    /// Chunk size in bytes for r3 files (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_size: Option<u32>,
}

/// Parity coding information for a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileParity {
    /// Erasure codec name.
    pub codec: String,
    /// Shard indices used for this file's data.
    pub shard_indices: Vec<u32>,
    /// Parity shard indices.
    pub parity_indices: Vec<u32>,
    /// Which shards contain parity data for this file.
    pub parity_in_shards: Vec<String>,
}

/// Byte-range access map for the content section.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentMap {
    /// Ordered list of blob regions within the content section.
    pub blob_regions: Vec<ContentBlobRegion>,
    /// Total content size in bytes.
    pub content_total_bytes: u64,
    /// Compression ratio achieved.
    pub compression_ratio: f64,
    /// Ratio of content that is encrypted.
    pub encrypted_ratio: f64,
}

/// A single blob region within the content section.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentBlobRegion {
    /// Unique identifier for this blob (e.g., "r0_file_abc123").
    pub id: String,
    /// Byte offset within the content section.
    pub offset: u64,
    /// Byte length of this blob.
    pub length: u64,
    /// Key tier used for encryption ("preview" or "content").
    pub key_tier: String,
    /// Compression algorithm applied (e.g., "webp-lossy", "lz4+zstd15+brotli11").
    pub compression: String,
}

/// Erasure coding metadata for a shard.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErasureMeta {
    /// The shard this metadata belongs to.
    pub shard_id: String,
    /// Erasure codec name (e.g., "clay-codes").
    pub erasure_codec: String,
    /// Erasure coding parameters.
    pub erasure_params: ErasureParams,
    /// Role of this shard (e.g., "data_shard_0").
    pub this_shard_role: String,
    /// Shard IDs that contain parity for this shard.
    pub parity_distributed_to: Vec<String>,
    /// Recovery threshold information.
    pub recovery_threshold: RecoveryThreshold,
    /// Fountain code configuration.
    pub fountain_config: FountainConfig,
    /// BLAKE3 hash of the entire shard content.
    pub shard_blake3: String,
}

/// Recovery threshold information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryThreshold {
    /// Number of data shards needed for reconstruction.
    pub data_shards_needed: u32,
    /// Total shards available.
    pub total_shards_available: u32,
    /// Which shards can be used for recovery.
    pub can_recover_with: Vec<String>,
}

/// Fountain code configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FountainConfig {
    /// Size of each symbol in bytes.
    pub symbol_size: u32,
    /// Number of source symbols.
    pub source_symbols: u32,
    /// Number of repair symbols per shard.
    pub repair_symbols_per_shard: u32,
    /// Minimum packets needed for recovery.
    pub min_packets_for_recovery: u32,
}

/// Erasure map within a shard index.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErasureMap {
    /// Clay code shards for erasure coding.
    pub clay_shards: Vec<ErasureClayShard>,
    /// Fountain code packets for erasure coding.
    pub fountain_packets: Vec<ErasureFountainPacket>,
}

/// A clay code erasure shard entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErasureClayShard {
    /// Index of this clay shard.
    pub shard_index: u32,
    /// Byte offset within the content section.
    pub content_offset: u64,
    /// Byte length of this shard.
    pub content_length: u64,
}

/// A fountain code erasure packet entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErasureFountainPacket {
    /// Packet sequence number.
    pub packet_id: u32,
    /// Byte offset within the content section.
    pub content_offset: u64,
    /// Byte length of this packet.
    pub content_length: u64,
}

/// Sprite sheet entry for batch thumbnail display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteSheetEntry {
    /// Byte offset of the sprite sheet in the content section.
    pub content_offset: u64,
    /// Byte length of the sprite sheet.
    pub content_length: u64,
    /// Grid layout (e.g., "4x4").
    pub grid: String,
    /// Number of thumbnails in this sprite sheet.
    pub thumb_count: u32,
}

/// Shard index within the root file.
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
