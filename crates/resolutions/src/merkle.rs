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
        let leaf_hex = leaf_hash.to_hex().to_string();

        // Find the expected leaf hash (hex string)
        let expected = match self.leaf_hashes.get(level) {
            Some(h) => h,
            None => return false,
        };

        if leaf_hex != *expected {
            return false;
        }

        // Walk the proof path — siblings are hex string bytes
        let mut current = leaf_hex.into_bytes();

        for step in &proof.path {
            match step {
                MerkleStep::Left(sibling) => {
                    let mut combined = Vec::with_capacity(sibling.len() + current.len());
                    combined.extend_from_slice(sibling);
                    combined.extend_from_slice(&current);
                    current = blake3::hash(&combined).to_hex().to_string().into_bytes();
                }
                MerkleStep::Right(sibling) => {
                    let mut combined = Vec::with_capacity(current.len() + sibling.len());
                    combined.extend_from_slice(&current);
                    combined.extend_from_slice(sibling);
                    current = blake3::hash(&combined).to_hex().to_string().into_bytes();
                }
            }
        }

        // Verify against root (hex string bytes)
        current == self.root_hash.as_bytes()
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
        let sibling_hash = self.leaf_hashes.get(*sibling_level)?.as_bytes().to_vec();

        if idx % 2 == 0 {
            path.push(MerkleStep::Right(sibling_hash));
        } else {
            path.push(MerkleStep::Left(sibling_hash));
        }

        // Level 1 (root): combine the two intermediate hashes
        let parent_idx = idx / 2;
        let inter_idx = if parent_idx == 0 { 1 } else { 0 };
        if let Some(root_sibling) = self.intermediate_hashes.get(inter_idx) {
            let root_sibling_bytes = root_sibling.as_bytes().to_vec();
            if parent_idx == 0 {
                path.push(MerkleStep::Right(root_sibling_bytes));
            } else {
                path.push(MerkleStep::Left(root_sibling_bytes));
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
    pub fn verify(&self, root_hash_hex: &str, leaf_hash_hex: &str) -> bool {
        let mut current = leaf_hash_hex.as_bytes().to_vec();

        for step in &self.path {
            match step {
                MerkleStep::Left(sibling) => {
                    let mut combined = Vec::with_capacity(sibling.len() + current.len());
                    combined.extend_from_slice(sibling);
                    combined.extend_from_slice(&current);
                    current = blake3::hash(&combined).to_hex().to_string().into_bytes();
                }
                MerkleStep::Right(sibling) => {
                    let mut combined = Vec::with_capacity(current.len() + sibling.len());
                    combined.extend_from_slice(&current);
                    combined.extend_from_slice(sibling);
                    current = blake3::hash(&combined).to_hex().to_string().into_bytes();
                }
            }
        }

        current == root_hash_hex.as_bytes()
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
        let leaf_hex = blake3::hash(b"medium_data").to_hex().to_string();
        assert!(proof.verify(&tree.root_hash, &leaf_hex));
    }
}

/// Sparse incremental Merkle tree supporting O(log n) updates.
///
/// For large libraries (100K+ files), rebuilding the entire tree is too slow.
/// This tree stores leaves and caches intermediate nodes, only recomputing
/// the path from updated leaf to root.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalMerkleTree {
    /// Leaf store: file_id → {resolution → hash}
    pub leaves: HashMap<String, HashMap<String, String>>,
    /// Cached intermediate nodes: node_path → hash
    pub cache: HashMap<String, String>,
    /// Height of the tree (ceil(log2(capacity)))
    pub height: usize,
    /// Capacity (maximum number of leaves)
    pub capacity: usize,
}

impl IncrementalMerkleTree {
    /// Create a new incremental Merkle tree with given capacity.
    pub fn new(capacity: usize) -> Self {
        let height = if capacity <= 1 {
            0
        } else {
            (capacity as f64).log2().ceil() as usize
        };
        Self {
            leaves: HashMap::new(),
            cache: HashMap::new(),
            height,
            capacity,
        }
    }

    /// Map a file_id to a deterministic leaf index using BLAKE3.
    pub fn file_id_to_leaf_index(&self, file_id: &str) -> usize {
        let hash = blake3::hash(file_id.as_bytes());
        let bytes = hash.as_bytes();
        let val = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        (val % self.capacity as u64) as usize
    }

    /// O(log n) update: update one file's resolution hashes and recompute path to root.
    pub fn update_file(&mut self, file_id: &str, resolution_hashes: &HashMap<String, String>) {
        let leaf_idx = self.file_id_to_leaf_index(file_id);
        self.leaves
            .insert(file_id.to_string(), resolution_hashes.clone());
        self.recompute_path(leaf_idx);
    }

    /// Recompute the path from a leaf index to the root.
    fn recompute_path(&mut self, leaf_idx: usize) {
        let mut idx = leaf_idx;
        for level in 0..self.height {
            let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            let node_key = format!("{}:{}", level, idx);
            let sibling_key = format!("{}:{}", level, sibling_idx);

            let left_hash = self.cache.get(&node_key).cloned().unwrap_or_default();
            let right_hash = self.cache.get(&sibling_key).cloned().unwrap_or_default();

            let combined = format!("{}||{}", left_hash, right_hash);
            let parent_hash = blake3::hash(combined.as_bytes()).to_hex().to_string();
            let parent_key = format!("{}:{}", level + 1, idx / 2);
            self.cache.insert(parent_key, parent_hash);

            idx /= 2;
        }
    }

    /// Get the current root hash.
    pub fn root(&self) -> String {
        if self.leaves.is_empty() {
            return blake3::hash(b"empty").to_hex().to_string();
        }
        self.cache
            .get(&format!("{}:0", self.height))
            .cloned()
            .unwrap_or_else(|| blake3::hash(b"empty").to_hex().to_string())
    }

    /// Generate a Merkle proof for one file's resolution.
    pub fn prove_file_resolution(&self, file_id: &str, _resolution: &str) -> Option<MerkleProof> {
        let leaf_idx = self.file_id_to_leaf_index(file_id);
        let mut path = Vec::new();
        let mut idx = leaf_idx;

        for level in 0..self.height {
            let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            let sibling_key = format!("{}:{}", level, sibling_idx);
            let sibling_bytes = self
                .cache
                .get(&sibling_key)
                .map(|h| h.as_bytes().to_vec())
                .unwrap_or_default();

            if idx % 2 == 0 {
                path.push(MerkleStep::Right(sibling_bytes));
            } else {
                path.push(MerkleStep::Left(sibling_bytes));
            }
            idx /= 2;
        }

        Some(MerkleProof { path })
    }
}
