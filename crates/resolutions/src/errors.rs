use thiserror::Error;

#[derive(Error, Debug)]
pub enum ResolutionError {
    #[error("Invalid magic bytes: expected {expected:?}, got {got:?}")]
    InvalidMagic { expected: String, got: String },

    #[error("Unsupported version: {0}")]
    UnsupportedVersion(String),

    #[error("Shard not found: {0}")]
    ShardNotFound(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid shard header: {0}")]
    InvalidShardHeader(String),

    #[error("Invalid root header: {0}")]
    InvalidRootHeader(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Invalid merkle proof: {0}")]
    InvalidMerkleProof(String),

    #[error("Integrity check failed: expected {expected}, got {got}")]
    IntegrityCheckFailed { expected: String, got: String },

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Codec error: {0}")]
    CodecError(String),

    #[error("Insufficient shards for recovery: need {need}, have {have}")]
    InsufficientShards { need: usize, have: usize },

    #[error("Invalid erasure parameters: k={k}, m={m}")]
    InvalidErasureParams { k: u32, m: u32 },

    #[error("Chunk not found: file {file_id}, chunk {chunk_index}")]
    ChunkNotFound { file_id: String, chunk_index: u32 },

    #[error("Invalid resolution level: {0}")]
    InvalidResolutionLevel(String),

    #[error("Blob region out of bounds: offset {offset} + length {length} exceeds content size {content_size}")]
    BlobRegionOutOfBounds {
        offset: u64,
        length: u64,
        content_size: u64,
    },

    #[error("Library already initialized")]
    LibraryAlreadyInitialized,

    #[error("Library not initialized: root.cybermanju not found")]
    LibraryNotInitialized,
}

pub type Result<T> = std::result::Result<T, ResolutionError>;
