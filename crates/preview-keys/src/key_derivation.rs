use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;

use crate::errors::PreviewKeyError;

/// Master key hierarchy — derives independent keys for each layer
pub struct KeyHierarchy {
    master_key: [u8; 32],
}

impl KeyHierarchy {
    pub fn new(master_key: [u8; 32]) -> Self {
        Self { master_key }
    }

    /// Derive the index encryption key for a library
    pub fn derive_index_key(&self, library_id: &str) -> [u8; 32] {
        let hk = Hkdf::<Sha256>::new(Some(b"cybermanju-index-v1"), &self.master_key);
        let mut key = [0u8; 32];
        hk.expand(library_id.as_bytes(), &mut key)
            .expect("HKDF expand failed");
        key
    }

    /// Derive the content encryption key for a file
    pub fn derive_content_key(&self, file_id: &str) -> [u8; 32] {
        let hk = Hkdf::<Sha256>::new(Some(b"cybermanju-content-v1"), &self.master_key);
        let mut key = [0u8; 32];
        hk.expand(file_id.as_bytes(), &mut key)
            .expect("HKDF expand failed");
        key
    }

    /// Derive the preview encryption key for a file
    pub fn derive_preview_key(&self, file_id: &str) -> [u8; 32] {
        let hk = Hkdf::<Sha256>::new(Some(b"cybermanju-preview-v1"), &self.master_key);
        let mut key = [0u8; 32];
        hk.expand(file_id.as_bytes(), &mut key)
            .expect("HKDF expand failed");
        key
    }

    /// Derive the view token signing key for a file+token combination
    pub fn derive_view_token_key(&self, file_id: &str, token_id: &str) -> [u8; 32] {
        let info = format!("{}:{}", file_id, token_id);
        let hk = Hkdf::<Sha256>::new(Some(b"cybermanju-view-token-v1"), &self.master_key);
        let mut key = [0u8; 32];
        hk.expand(info.as_bytes(), &mut key)
            .expect("HKDF expand failed");
        key
    }
}

/// Generate a random 32-byte master key
pub fn generate_master_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

/// Encrypt index data with AES-256-GCM
pub fn encrypt_index(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, PreviewKeyError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| PreviewKeyError::EncryptionFailed(e.to_string()))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let mut ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| PreviewKeyError::EncryptionFailed(e.to_string()))?;

    // Prepend nonce
    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.append(&mut ciphertext);
    Ok(result)
}

/// Decrypt index data with AES-256-GCM
pub fn decrypt_index(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, PreviewKeyError> {
    if data.len() < 12 {
        return Err(PreviewKeyError::DecryptionFailed("data too short".into()));
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| PreviewKeyError::DecryptionFailed(e.to_string()))?;

    let nonce = Nonce::from_slice(&data[..12]);
    cipher
        .decrypt(nonce, &data[12..])
        .map_err(|e| PreviewKeyError::DecryptionFailed(e.to_string()))
}

/// Encrypt preview data with ChaCha20-Poly1305, nonce derived from file_id
pub fn encrypt_preview(
    data: &[u8],
    key: &[u8; 32],
    file_id: &str,
) -> Result<Vec<u8>, PreviewKeyError> {
    let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));

    // Derive nonce from file_id: BLAKE3(file_id)[0..12]
    let hash = blake3::hash(file_id.as_bytes());
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes.copy_from_slice(&hash.as_bytes()[..12]);
    let nonce = ChaChaNonce::from_slice(&nonce_bytes);

    let mut ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| PreviewKeyError::EncryptionFailed(e.to_string()))?;

    // Prepend nonce
    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.append(&mut ciphertext);
    Ok(result)
}

/// Decrypt preview data with ChaCha20-Poly1305
pub fn decrypt_preview(
    data: &[u8],
    key: &[u8; 32],
    file_id: &str,
) -> Result<Vec<u8>, PreviewKeyError> {
    if data.len() < 12 {
        return Err(PreviewKeyError::DecryptionFailed("data too short".into()));
    }

    let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));

    let hash = blake3::hash(file_id.as_bytes());
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes.copy_from_slice(&hash.as_bytes()[..12]);
    let nonce = ChaChaNonce::from_slice(&nonce_bytes);

    cipher
        .decrypt(nonce, &data[12..])
        .map_err(|e| PreviewKeyError::DecryptionFailed(e.to_string()))
}

/// Encrypt a content chunk with ChaCha20-Poly1305, per-chunk nonce
pub fn encrypt_content(
    data: &[u8],
    key: &[u8; 32],
    file_id: &str,
    chunk_index: u32,
) -> Result<Vec<u8>, PreviewKeyError> {
    let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));

    // Per-chunk nonce: BLAKE3(file_id || chunk_index)[0..12]
    let mut hasher = blake3::Hasher::new();
    hasher.update(file_id.as_bytes());
    hasher.update(&chunk_index.to_le_bytes());
    let hash = hasher.finalize();
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes.copy_from_slice(&hash.as_bytes()[..12]);
    let nonce = ChaChaNonce::from_slice(&nonce_bytes);

    let mut ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| PreviewKeyError::EncryptionFailed(e.to_string()))?;

    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.append(&mut ciphertext);
    Ok(result)
}

/// Decrypt a content chunk with ChaCha20-Poly1305
pub fn decrypt_content(
    data: &[u8],
    key: &[u8; 32],
    file_id: &str,
    chunk_index: u32,
) -> Result<Vec<u8>, PreviewKeyError> {
    if data.len() < 12 {
        return Err(PreviewKeyError::DecryptionFailed("data too short".into()));
    }

    let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));

    let mut hasher = blake3::Hasher::new();
    hasher.update(file_id.as_bytes());
    hasher.update(&chunk_index.to_le_bytes());
    let hash = hasher.finalize();
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes.copy_from_slice(&hash.as_bytes()[..12]);
    let nonce = ChaChaNonce::from_slice(&nonce_bytes);

    cipher
        .decrypt(nonce, &data[12..])
        .map_err(|e| PreviewKeyError::DecryptionFailed(e.to_string()))
}

/// Derive the shard MAC key from master key using HKDF.
pub fn derive_shard_mac_key(master_key: &[u8; 32], shard_id: &str) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(Some(b"cybermanju-shard-mac-v1"), master_key);
    let mut key = [0u8; 32];
    hk.expand(shard_id.as_bytes(), &mut key)
        .expect("HKDF expand failed for shard MAC key");
    key
}

/// Compute a keyed shard MAC: BLAKE3-keyed(shard_mac_key, content).
pub fn compute_shard_mac(content: &[u8], shard_id: &str, master_key: &[u8; 32]) -> [u8; 32] {
    let mac_key = derive_shard_mac_key(master_key, shard_id);
    *blake3::keyed_hash(&mac_key, content).as_bytes()
}

/// Verify a shard MAC with constant-time comparison.
pub fn verify_shard_mac(
    content: &[u8],
    shard_id: &str,
    master_key: &[u8; 32],
    expected: &[u8; 32],
) -> bool {
    use subtle::ConstantTimeEq;
    let computed = compute_shard_mac(content, shard_id, master_key);
    computed.ct_eq(expected).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation() {
        let key = generate_master_key();
        let hierarchy = KeyHierarchy::new(key);

        let idx1 = hierarchy.derive_index_key("lib1");
        let idx2 = hierarchy.derive_index_key("lib2");
        assert_ne!(idx1, idx2);

        let content = hierarchy.derive_content_key("file1");
        let preview = hierarchy.derive_preview_key("file1");
        assert_ne!(content, preview);
    }

    #[test]
    fn test_index_encrypt_decrypt() {
        let key = generate_master_key();
        let data = b"Hello, encrypted index!";
        let encrypted = encrypt_index(data, &key).unwrap();
        let decrypted = decrypt_index(&encrypted, &key).unwrap();
        assert_eq!(&decrypted, data);
    }

    #[test]
    fn test_preview_encrypt_decrypt() {
        let key = generate_master_key();
        let data = b"Preview image data";
        let encrypted = encrypt_preview(data, &key, "file_123").unwrap();
        let decrypted = decrypt_preview(&encrypted, &key, "file_123").unwrap();
        assert_eq!(&decrypted, data);
    }

    #[test]
    fn test_content_chunk_encrypt_decrypt() {
        let key = generate_master_key();
        let data = b"Chunk data for content";
        let encrypted = encrypt_content(data, &key, "file_456", 0).unwrap();
        let decrypted = decrypt_content(&encrypted, &key, "file_456", 0).unwrap();
        assert_eq!(&decrypted, data);
    }
}
