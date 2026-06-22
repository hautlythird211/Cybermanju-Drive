use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A BLAKE3-based Merkle tree for resolution verification.
///
/// Each file has a Merkle tree where the root is the file's identity and leaves
/// are individual resolutions (r0, r1, r2, r3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionMerkleTree {
    /// The root hash of the entire Merkle tree.
    pub root_hash: String,
    /// Leaf hashes keyed by resolution level (r0, r1, r2, r3).
    pub leaf_hashes: HashMap<String, String>,
    /// Intermediate hashes for verification.
    pub intermediate_hashes: Vec<String>,
}

impl ResolutionMerkleTree {
    /// Build a Merkle tree from resolution hashes.
    ///
    /// Tree structure:
    /// ```text
    ///                     File Merkle Root (BLAKE3)
    ///                    /                          \
    ///           BLAKE3(r0_hash || r1_hash)    BLAKE3(r2_hash || r3_hash)
    ///            /            \                /              \
    ///         r0_hash      r1_hash         r2_hash          r3_hash
    ///         (200x200)   (640x480)       (1920x1080)     (original)
    /// ```
    pub fn build(resolution_hashes: &HashMap<String, String>) -> Self {
        let mut leaf_hashes = HashMap::new();

        // Ensure r0, r1, r2, r3 are present with defaults if missing
        for level in &["r0", "r1", "r2", "r3"] {
            let hash = resolution_hashes
                .get(*level)
                .cloned()
                .unwrap_or_else(|| blake3::hash(b"").to_hex().to_string());
            leaf_hashes.insert(level.to_string(), hash);
        }

        let r0 = leaf_hashes.get("r0").unwrap();
        let r1 = leaf_hashes.get("r1").unwrap();
        let r2 = leaf_hashes.get("r2").unwrap();
        let r3 = leaf_hashes.get("r3").unwrap();

        let left = Self::hash_pair(r0.as_bytes(), r1.as_bytes());
        let right = Self::hash_pair(r2.as_bytes(), r3.as_bytes());

        let intermediate_hashes = vec![left.clone(), right.clone()];

        let root_hash = Self::hash_pair(left.as_bytes(), right.as_bytes());

        Self {
            root_hash,
            leaf_hashes,
            intermediate_hashes,
        }
    }

    /// Compute the BLAKE3 hash of two concatenated hashes.
    pub fn hash_pair(left: &[u8], right: &[u8]) -> String {
        let mut combined = Vec::with_capacity(left.len() + right.len());
        combined.extend_from_slice(left);
        combined.extend_from_slice(right);
        blake3::hash(&combined).to_hex().to_string()
    }

    /// Compute a BLAKE3 hash of raw data.
    pub fn hash_data(data: &[u8]) -> String {
        blake3::hash(data).to_hex().to_string()
    }

    /// Verify that a resolution leaf is part of this tree.
    pub fn verify_leaf(&self, level: &str, data: &[u8], proof: &MerkleProof) -> bool {
        let leaf_hash = blake3::hash(data);

        // Find the expected leaf hash
        let expected = match self.leaf_hashes.get(level) {
            Some(h) => h.as_bytes().to_vec(),
            None => return false,
        };

        if leaf_hash.as_bytes() != expected.as_slice() {
            return false;
        }

        // Walk the proof path
        let mut current = leaf_hash.as_bytes().to_vec();

        for step in &proof.path {
            match step {
                MerkleStep::Left(sibling) => {
                    current = blake3::hashv(&[sibling, &current]).as_bytes().to_vec();
                }
                MerkleStep::Right(sibling) => {
                    current = blake3::hashv(&[&current, sibling]).as_bytes().to_vec();
                }
            }
        }

        // Verify against root
        current.as_slice() == hex::decode(&self.root_hash).unwrap_or_default()
    }

    /// Generate a Merkle proof for a specific resolution level.
    pub fn prove(&self, level: &str) -> Option<MerkleProof> {
        let levels = ["r0", "r1", "r2", "r3"];
        let idx = levels.iter().position(|l| *l == level)?;
        let mut path = Vec::new();

        // For a 4-leaf tree:
        // r0 at index 0, r1 at index 1 → left pair
        // r2 at index 2, r3 at index 3 → right pair
        // Intermediate: [left, right]

        // Level 0 (pairs): combine adjacent leaves
        let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
        let sibling_level = levels.get(sibling_idx)?;
        let sibling_hash = self.leaf_hashes.get(sibling_level)?.as_bytes().to_vec();

        if idx % 2 == 0 {
            path.push(MerkleStep::Right(sibling_hash));
        } else {
            path.push(MerkleStep::Left(sibling_hash));
        }

        // Level 1 (root): combine the two intermediate hashes
        let parent_idx = idx / 2;
        let inter_idx = if parent_idx == 0 { 1 } else { 0 };
        if let Some(root_sibling) = self.intermediate_hashes.get(inter_idx) {
            if parent_idx == 0 {
                path.push(MerkleStep::Right(root_sibling.as_bytes().to_vec()));
            } else {
                path.push(MerkleStep::Left(root_sibling.as_bytes().to_vec()));
            }
        }

        Some(MerkleProof { path })
    }

    /// Get the root hash as a hex string.
    pub fn root_hex(&self) -> &str {
        &self.root_hash
    }

    /// Verify the tree integrity by recomputing the root.
    pub fn verify_integrity(&self) -> bool {
        let r0 = self.leaf_hashes.get("r0").unwrap();
        let r1 = self.leaf_hashes.get("r1").unwrap();
        let r2 = self.leaf_hashes.get("r2").unwrap();
        let r3 = self.leaf_hashes.get("r3").unwrap();

        let left = Self::hash_pair(r0.as_bytes(), r1.as_bytes());
        let right = Self::hash_pair(r2.as_bytes(), r3.as_bytes());
        let recomputed_root = Self::hash_pair(left.as_bytes(), right.as_bytes());

        recomputed_root == self.root_hash
    }
}

/// A step in a Merkle proof path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MerkleStep {
    /// The sibling is to the left of the current node.
    Left(Vec<u8>),
    /// The sibling is to the right of the current node.
    Right(Vec<u8>),
}

/// A Merkle proof for verifying a leaf is part of the tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    /// The path from leaf to root.
    pub path: Vec<MerkleStep>,
}

impl MerkleProof {
    /// Verify this proof against a root hash and leaf hash.
    pub fn verify(&self, root_hash: &[u8], leaf_hash: &[u8]) -> bool {
        let mut current = leaf_hash.to_vec();

        for step in &self.path {
            match step {
                MerkleStep::Left(sibling) => {
                    current = blake3::hashv(&[sibling, &current]).as_bytes().to_vec();
                }
                MerkleStep::Right(sibling) => {
                    current = blake3::hashv(&[&current, sibling]).as_bytes().to_vec();
                }
            }
        }

        current.as_slice() == root_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_hashes() -> HashMap<String, String> {
        let mut h = HashMap::new();
        h.insert(
            "r0".into(),
            blake3::hash(b"thumb_data").to_hex().to_string(),
        );
        h.insert(
            "r1".into(),
            blake3::hash(b"preview_data").to_hex().to_string(),
        );
        h.insert(
            "r2".into(),
            blake3::hash(b"medium_data").to_hex().to_string(),
        );
        h.insert(
            "r3".into(),
            blake3::hash(b"original_data").to_hex().to_string(),
        );
        h
    }

    #[test]
    fn build_tree_and_verify() {
        let hashes = sample_hashes();
        let tree = ResolutionMerkleTree::build(&hashes);

        assert!(tree.verify_integrity());

        // Verify each leaf
        let proof_r0 = tree.prove("r0").unwrap();
        assert!(tree.verify_leaf("r0", b"thumb_data", &proof_r0));

        let proof_r1 = tree.prove("r1").unwrap();
        assert!(tree.verify_leaf("r1", b"preview_data", &proof_r1));

        let proof_r2 = tree.prove("r2").unwrap();
        assert!(tree.verify_leaf("r2", b"medium_data", &proof_r2));

        let proof_r3 = tree.prove("r3").unwrap();
        assert!(tree.verify_leaf("r3", b"original_data", &proof_r3));
    }

    #[test]
    fn wrong_data_fails_verification() {
        let hashes = sample_hashes();
        let tree = ResolutionMerkleTree::build(&hashes);

        let proof = tree.prove("r0").unwrap();
        assert!(!tree.verify_leaf("r0", b"wrong_data", &proof));
    }

    #[test]
    fn hash_pair_is_deterministic() {
        let a = blake3::hash(b"a").as_bytes().to_vec();
        let b = blake3::hash(b"b").as_bytes().to_vec();

        let h1 = ResolutionMerkleTree::hash_pair(&a, &b);
        let h2 = ResolutionMerkleTree::hash_pair(&a, &b);
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_pair_order_matters() {
        let a = blake3::hash(b"a").as_bytes().to_vec();
        let b = blake3::hash(b"b").as_bytes().to_vec();

        let h1 = ResolutionMerkleTree::hash_pair(&a, &b);
        let h2 = ResolutionMerkleTree::hash_pair(&b, &a);
        assert_ne!(h1, h2);
    }

    #[test]
    fn proof_verifies_at_root() {
        let hashes = sample_hashes();
        let tree = ResolutionMerkleTree::build(&hashes);

        let proof = tree.prove("r2").unwrap();
        let leaf = blake3::hash(b"medium_data");
        let root = hex::decode(&tree.root_hash).unwrap();
        assert!(proof.verify(&root, leaf.as_bytes()));
    }
}
