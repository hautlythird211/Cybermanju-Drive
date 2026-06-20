use thiserror::Error;

#[derive(Debug, Error)]
pub enum RecoveryError {
    #[error("image error: {0}")]
    ImageError(String),

    #[error("upscaling error: {0}")]
    UpscalingError(String),

    #[error("reconstruction error: {0}")]
    ReconstructionError(String),

    #[error("network error: {0}")]
    NetworkError(String),

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("invalid format: {0}")]
    InvalidFormat(String),

    #[error("missing data: {0}")]
    MissingData(String),
}
