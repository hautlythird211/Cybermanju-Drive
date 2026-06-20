use thiserror::Error;

#[derive(Debug, Error)]
pub enum ErasureError {
    #[error("too few shards: need at least {0}")]
    TooFewShards(usize),

    #[error("reconstruction failed: {0}")]
    ReconstructionFailed(String),

    #[error("invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("encoding failed: {0}")]
    EncodingFailed(String),

    #[error("decoding failed: {0}")]
    DecodingFailed(String),

    #[error("shamir error: {0}")]
    ShamirError(String),

    #[error("fountain error: {0}")]
    FountainError(String),
}
