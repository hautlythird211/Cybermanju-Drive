use thiserror::Error;

#[derive(Debug, Error)]
pub enum PreviewKeyError {
    #[error("invalid key length: expected {expected}, got {got}")]
    InvalidKeyLength { expected: usize, got: usize },

    #[error("encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("view token has expired")]
    TokenExpired,

    #[error("view token has been revoked")]
    TokenRevoked,

    #[error("invalid view token: {0}")]
    InvalidToken(String),

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("HKDF error: {0}")]
    HkdfError(String),

    #[error("crypto error: {0}")]
    CryptoError(String),
}
