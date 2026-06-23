use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::errors::PreviewKeyError;

/// A Merkle accumulator-based Certificate Revocation List for view tokens.
/// Append-only: each revocation adds a new leaf, root changes via O(log n) update.
/// Verifiers check: is token_id in the accumulator?
#[derive(Debug, Clone)]
pub struct MerkleAccumulatorCRL {
    /// Append-only list of revoked token IDs
    revoked_ids: Vec<String>,
    /// Cached leaf hashes: index → BLAKE3(leaf_id)
    leaf_hashes: Vec<[u8; 32]>,
    /// Cached intermediate nodes: path → hash
    cache: HashMap<Vec<usize>, [u8; 32]>,
    /// Current Merkle root
    root: [u8; 32],
    /// Height of the tree (ceil(log2(2*capacity)))
    height: usize,
}

/// Serialization helper for MerkleAccumulatorCRL.
#[derive(Serialize, Deserialize)]
struct MerkleAccumulatorCRLSerialized {
    revoked_ids: Vec<String>,
    leaf_hashes_hex: Vec<String>,
    root_hex: String,
    height: usize,
}

impl Serialize for MerkleAccumulatorCRL {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let ser = MerkleAccumulatorCRLSerialized {
            revoked_ids: self.revoked_ids.clone(),
            leaf_hashes_hex: self.leaf_hashes.iter().map(|h| hex::encode(h)).collect(),
            root_hex: hex::encode(self.root),
            height: self.height,
        };
        ser.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MerkleAccumulatorCRL {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let ser = MerkleAccumulatorCRLSerialized::deserialize(deserializer)?;
        let leaf_hashes: Result<Vec<[u8; 32]>, _> = ser
            .leaf_hashes_hex
            .iter()
            .map(|h| {
                let bytes = hex::decode(h).map_err(serde::de::Error::custom)?;
                let mut arr = [0u8; 32];
                if bytes.len() != 32 {
                    return Err(serde::de::Error::custom("leaf hash must be 32 bytes"));
                }
                arr.copy_from_slice(&bytes);
                Ok(arr)
            })
            .collect();
        let root_bytes = hex::decode(&ser.root_hex).map_err(serde::de::Error::custom)?;
        let mut root = [0u8; 32];
        if root_bytes.len() != 32 {
            return Err(serde::de::Error::custom("root must be 32 bytes"));
        }
        root.copy_from_slice(&root_bytes);

        // Rebuild cache from leaf hashes
        let mut crl = MerkleAccumulatorCRL {
            revoked_ids: ser.revoked_ids,
            leaf_hashes: leaf_hashes?,
            cache: HashMap::new(),
            root,
            height: ser.height,
        };
        // Rebuild the cache by re-hashing the tree
        crl.rebuild_cache();
        Ok(crl)
    }
}

/// Merkle proof for a single revoked token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevocationProof {
    pub token_id: String,
    pub leaf_index: usize,
    pub leaf_hash: String,
    pub sibling_hashes: Vec<String>,
    pub path_bits: Vec<bool>,
    pub root: String,
}

impl MerkleAccumulatorCRL {
    /// Create an empty CRL accumulator.
    pub fn new() -> Self {
        let height = 20; // supports up to 2^20 ≈ 1M revocations
        let empty_leaf = blake3::hash(b"EMPTY");
        let root = *empty_leaf.as_bytes();

        // Build default tree with empty leaves
        let mut cache = HashMap::new();
        let default_empty = *blake3::hash(b"EMPTY").as_bytes();
        let mut level_size = 1usize << height;

        for h in 0..height {
            for i in 0..level_size {
                let path = vec![h, i];
                cache.insert(path, default_empty);
            }
            level_size /= 2;
        }

        Self {
            revoked_ids: Vec::new(),
            leaf_hashes: Vec::new(),
            cache,
            root,
            height,
        }
    }

    /// Append a revocation — O(height) root update.
    pub fn revoke(&mut self, token_id: &str) -> Result<RevocationProof, PreviewKeyError> {
        if self.revoked_ids.contains(&token_id.to_string()) {
            return Err(PreviewKeyError::SerializationError(
                "token already revoked".into(),
            ));
        }

        let leaf_idx = self.revoked_ids.len();
        self.revoked_ids.push(token_id.to_string());

        // Compute leaf hash: BLAKE3(leaf_id_bytes)
        let leaf_hash = *blake3::hash(token_id.as_bytes()).as_bytes();
        self.leaf_hashes.push(leaf_hash);

        // Update the Merkle tree path from this leaf to root
        self.update_path(leaf_idx);

        // Generate proof
        let proof = self.prove_leaf(leaf_idx);

        Ok(proof)
    }

    /// Check membership — O(height) via Merkle proof.
    pub fn is_revoked(
        &self,
        token_id: &str,
        proof: &RevocationProof,
    ) -> Result<bool, PreviewKeyError> {
        // Reconstruct leaf hash
        let leaf_hash = *blake3::hash(token_id.as_bytes()).as_bytes();

        // Verify leaf hash matches proof
        let proof_leaf = hex::decode(&proof.leaf_hash)
            .map_err(|_| PreviewKeyError::SerializationError("invalid proof leaf hash".into()))?;
        if leaf_hash[..] != proof_leaf[..] {
            return Ok(false);
        }

        // Walk up the tree
        let mut current = leaf_hash;

        for (i, sibling_hex) in proof.sibling_hashes.iter().enumerate() {
            let sibling = hex::decode(sibling_hex).map_err(|_| {
                PreviewKeyError::SerializationError(format!("invalid sibling hash at level {}", i))
            })?;

            let mut hasher = Hasher::new();
            if proof.path_bits[i] {
                hasher.update(&sibling);
                hasher.update(&current);
            } else {
                hasher.update(&current);
                hasher.update(&sibling);
            }
            current = *hasher.finalize().as_bytes();
        }

        // Verify root matches
        let expected_root = hex::decode(&proof.root)
            .map_err(|_| PreviewKeyError::SerializationError("invalid proof root".into()))?;
        Ok(current[..] == expected_root[..])
    }

    /// Get the current Merkle root.
    pub fn root_hex(&self) -> String {
        hex::encode(self.root)
    }

    /// Get number of revoked tokens.
    pub fn len(&self) -> usize {
        self.revoked_ids.len()
    }

    /// Check if accumulator is empty.
    pub fn is_empty(&self) -> bool {
        self.revoked_ids.is_empty()
    }

    /// Get all revoked token IDs.
    pub fn revoked_ids(&self) -> &[String] {
        &self.revoked_ids
    }

    /// Update the Merkle tree path from a leaf to the root.
    fn update_path(&mut self, leaf_idx: usize) {
        let mut idx = leaf_idx;
        let mut current_hash = self.leaf_hashes[leaf_idx];

        for h in 0..self.height {
            let is_right = idx & 1 == 1;
            let sibling_idx = if is_right { idx - 1 } else { idx + 1 };

            let sibling_hash = if sibling_idx < self.leaf_hashes.len() {
                self.leaf_hashes[sibling_idx]
            } else {
                *blake3::hash(b"EMPTY").as_bytes()
            };

            let mut hasher = Hasher::new();
            if is_right {
                hasher.update(&sibling_hash);
                hasher.update(&current_hash);
            } else {
                hasher.update(&current_hash);
                hasher.update(&sibling_hash);
            }
            current_hash = *hasher.finalize().as_bytes();

            let path = vec![h, idx / 2];
            self.cache.insert(path, current_hash);

            idx /= 2;
        }

        self.root = current_hash;
    }

    /// Generate a Merkle proof for a leaf.
    fn prove_leaf(&self, leaf_idx: usize) -> RevocationProof {
        let mut sibling_hashes = Vec::new();
        let mut path_bits = Vec::new();
        let mut idx = leaf_idx;

        for _ in 0..self.height {
            let is_right = idx & 1 == 1;
            let sibling_idx = if is_right { idx - 1 } else { idx + 1 };

            let sibling_hash = if sibling_idx < self.leaf_hashes.len() {
                self.leaf_hashes[sibling_idx]
            } else {
                *blake3::hash(b"EMPTY").as_bytes()
            };

            sibling_hashes.push(hex::encode(sibling_hash));
            path_bits.push(is_right);

            idx /= 2;
        }

        RevocationProof {
            token_id: self.revoked_ids[leaf_idx].clone(),
            leaf_index: leaf_idx,
            leaf_hash: hex::encode(self.leaf_hashes[leaf_idx]),
            sibling_hashes,
            path_bits,
            root: self.root_hex(),
        }
    }

    /// Serialize the accumulator to bytes for storage.
    pub fn to_bytes(&self) -> Result<Vec<u8>, PreviewKeyError> {
        serde_json::to_vec(self).map_err(|e| PreviewKeyError::SerializationError(e.to_string()))
    }

    /// Deserialize the accumulator from bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self, PreviewKeyError> {
        serde_json::from_slice(data).map_err(|e| PreviewKeyError::SerializationError(e.to_string()))
    }

    /// Rebuild the internal cache from leaf hashes.
    fn rebuild_cache(&mut self) {
        self.cache.clear();
        let default_empty = *blake3::hash(b"EMPTY").as_bytes();
        let mut level_size = 1usize << self.height;

        // Copy leaf hashes into the cache
        for (i, leaf) in self.leaf_hashes.iter().enumerate() {
            self.cache.insert(vec![0, i], *leaf);
        }

        // Fill remaining leaf positions with empty hash
        for i in self.leaf_hashes.len()..level_size {
            self.cache.insert(vec![0, i], default_empty);
        }

        // Build up the tree
        let mut prev_level_size = level_size;
        level_size /= 2;
        for h in 1..=self.height {
            for i in 0..level_size {
                let left_path = vec![h - 1, i * 2];
                let right_path = vec![h - 1, i * 2 + 1];
                let left = self.cache.get(&left_path).copied().unwrap_or(default_empty);
                let right = self
                    .cache
                    .get(&right_path)
                    .copied()
                    .unwrap_or(default_empty);

                let mut hasher = Hasher::new();
                hasher.update(&left);
                hasher.update(&right);
                let parent = *hasher.finalize().as_bytes();
                self.cache.insert(vec![h, i], parent);
            }
            prev_level_size = level_size;
            level_size /= 2;
        }
    }
}

impl Default for MerkleAccumulatorCRL {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_crl() {
        let crl = MerkleAccumulatorCRL::new();
        assert!(crl.is_empty());
        assert_eq!(crl.len(), 0);
        assert!(!crl.root_hex().is_empty());
    }

    #[test]
    fn test_revoke_token() {
        let mut crl = MerkleAccumulatorCRL::new();
        let proof = crl.revoke("token1").unwrap();
        assert_eq!(crl.len(), 1);
        assert_eq!(proof.token_id, "token1");
        assert!(!crl.root_hex().is_empty());
    }

    #[test]
    fn test_revoke_duplicate() {
        let mut crl = MerkleAccumulatorCRL::new();
        crl.revoke("token1").unwrap();
        assert!(crl.revoke("token1").is_err());
    }

    #[test]
    fn test_is_revoked_valid_proof() {
        let mut crl = MerkleAccumulatorCRL::new();
        let proof = crl.revoke("token1").unwrap();
        assert!(crl.is_revoked("token1", &proof).unwrap());
    }

    #[test]
    fn test_is_revoked_wrong_token() {
        let mut crl = MerkleAccumulatorCRL::new();
        let proof = crl.revoke("token1").unwrap();
        assert!(!crl.is_revoked("token2", &proof).unwrap());
    }

    #[test]
    fn test_multiple_revocations() {
        let mut crl = MerkleAccumulatorCRL::new();
        let proof1 = crl.revoke("token1").unwrap();
        let proof2 = crl.revoke("token2").unwrap();
        let proof3 = crl.revoke("token3").unwrap();

        assert_eq!(crl.len(), 3);
        assert!(crl.is_revoked("token1", &proof1).unwrap());
        assert!(crl.is_revoked("token2", &proof2).unwrap());
        assert!(crl.is_revoked("token3", &proof3).unwrap());
        assert!(!crl.is_revoked("token1", &proof2).unwrap());
    }

    #[test]
    fn test_root_changes_with_revocations() {
        let mut crl = MerkleAccumulatorCRL::new();
        let root0 = crl.root_hex();
        crl.revoke("token1").unwrap();
        let root1 = crl.root_hex();
        crl.revoke("token2").unwrap();
        let root2 = crl.root_hex();

        assert_ne!(root0, root1);
        assert_ne!(root1, root2);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut crl = MerkleAccumulatorCRL::new();
        crl.revoke("token1").unwrap();
        crl.revoke("token2").unwrap();

        let bytes = crl.to_bytes().unwrap();
        let restored = MerkleAccumulatorCRL::from_bytes(&bytes).unwrap();

        assert_eq!(crl.len(), restored.len());
        assert_eq!(crl.root_hex(), restored.root_hex());
        assert_eq!(crl.revoked_ids(), restored.revoked_ids());
    }

    #[test]
    fn test_proof_verifies_after_serialization() {
        let mut crl = MerkleAccumulatorCRL::new();
        crl.revoke("token1").unwrap();
        let proof = crl.revoke("token2").unwrap();

        let bytes = crl.to_bytes().unwrap();
        let restored = MerkleAccumulatorCRL::from_bytes(&bytes).unwrap();

        assert!(restored.is_revoked("token2", &proof).unwrap());
    }
}
