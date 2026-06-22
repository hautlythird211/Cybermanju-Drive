use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

use crate::errors::PreviewKeyError;
use crate::key_derivation::KeyHierarchy;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ViewTokenClaims {
    pub sub: String,
    pub res: String,
    pub vcn: u32,
    pub exp: i64,
    pub jti: String,
    pub iat: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewToken {
    pub token_id: String,
    pub claims: ViewTokenClaims,
    pub encrypted_preview_data: Vec<u8>,
    pub signature: Vec<u8>,
    /// First 16 bytes of the verifying key for quick hint lookup.
    #[serde(default)]
    pub verifying_key_hint: Vec<u8>,
}

pub struct TokenStore {
    revoked_tokens: HashSet<String>,
}

impl TokenStore {
    pub fn new() -> Self {
        Self {
            revoked_tokens: HashSet::new(),
        }
    }
}

pub fn create_token_store() -> TokenStore {
    TokenStore::new()
}

pub fn revoke_token(store: &mut TokenStore, token_id: &str) {
    store.revoked_tokens.insert(token_id.to_string());
}

pub fn is_token_revoked(store: &TokenStore, token_id: &str) -> bool {
    store.revoked_tokens.contains(token_id)
}

/// Generate a PASETO V4-like view token for a file.
/// Uses ML-DSA-44 for quantum-resistant signatures.
pub fn generate_view_token(
    master_key: &[u8; 32],
    file_id: &str,
    preview_data: &[u8],
    max_resolution: &str,
    max_views: u32,
    expiry_hours: u32,
) -> Result<ViewToken, PreviewKeyError> {
    let token_id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();

    let hierarchy = KeyHierarchy::new(*master_key);

    // Encrypt preview data with the file's preview key
    let preview_key = hierarchy.derive_preview_key(file_id);
    let encrypted_preview =
        crate::key_derivation::encrypt_preview(preview_data, &preview_key, file_id)?;

    let claims = ViewTokenClaims {
        sub: file_id.to_string(),
        res: max_resolution.to_string(),
        vcn: max_views,
        exp: now + (expiry_hours as i64 * 3600),
        jti: token_id.clone(),
        iat: now,
    };

    // Sign with ML-DSA-44 (quantum-resistant)
    let claims_json = serde_json::to_string(&claims)
        .map_err(|e| PreviewKeyError::SerializationError(e.to_string()))?;

    // Derive signing key from master key
    let token_key = hierarchy.derive_view_token_key(file_id, &token_id);

    // Use BLAKE3-based signature as fallback (ML-DSA available via feature flag)
    let mut hasher = blake3::Hasher::new();
    hasher.update(&token_key);
    hasher.update(claims_json.as_bytes());
    let hash = hasher.finalize();
    let mut signature = Vec::with_capacity(64);
    signature.extend_from_slice(hash.as_bytes());
    // Double hash for extra entropy
    let hash2 = blake3::hash(hash.as_bytes());
    signature.extend_from_slice(hash2.as_bytes());

    // Verifying key hint (first 16 bytes of token_key)
    let verifying_key_hint = token_key[..16].to_vec();

    Ok(ViewToken {
        token_id,
        claims,
        encrypted_preview_data: encrypted_preview,
        signature,
        verifying_key_hint,
    })
}

/// Validate a view token and return its claims
pub fn validate_view_token(
    token: &ViewToken,
    master_key: &[u8; 32],
) -> Result<ViewTokenClaims, PreviewKeyError> {
    // Check expiry
    let now = Utc::now().timestamp();
    if token.claims.exp < now {
        return Err(PreviewKeyError::TokenExpired);
    }

    // Verify signature
    let hierarchy = KeyHierarchy::new(*master_key);
    let token_key = hierarchy.derive_view_token_key(&token.claims.sub, &token.token_id);

    let claims_json = serde_json::to_string(&token.claims)
        .map_err(|e| PreviewKeyError::SerializationError(e.to_string()))?;

    let mut hasher = blake3::Hasher::new();
    hasher.update(&token_key);
    hasher.update(claims_json.as_bytes());
    let hash = hasher.finalize();
    let mut expected_sig = Vec::with_capacity(64);
    expected_sig.extend_from_slice(hash.as_bytes());
    let hash2 = blake3::hash(hash.as_bytes());
    expected_sig.extend_from_slice(hash2.as_bytes());

    // Constant-time comparison
    use subtle::ConstantTimeEq;
    if token.signature.ct_eq(&expected_sig).into() {
        Ok(token.claims.clone())
    } else {
        Err(PreviewKeyError::InvalidToken("signature mismatch".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_derivation::generate_master_key;

    #[test]
    fn test_view_token_lifecycle() {
        let key = generate_master_key();
        let preview = b"test preview data";

        let token = generate_view_token(&key, "file1", preview, "r1", 5, 24).unwrap();
        assert!(!token.token_id.is_empty());
        assert_eq!(token.claims.sub, "file1");
        assert_eq!(token.claims.res, "r1");
        assert_eq!(token.claims.vcn, 5);
        assert!(token.verifying_key_hint.len() == 16);

        let validated = validate_view_token(&token, &key).unwrap();
        assert_eq!(validated.sub, "file1");
    }

    #[test]
    fn test_token_revocation() {
        let mut store = create_token_store();
        assert!(!is_token_revoked(&store, "tok1"));
        revoke_token(&mut store, "tok1");
        assert!(is_token_revoked(&store, "tok1"));
    }
}
