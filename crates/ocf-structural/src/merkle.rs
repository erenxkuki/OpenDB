use ocf_core::content_hash;

/// Binary Merkle tree over 32-byte leaf hashes.
/// Leaves are content hashes; internal nodes = BLAKE3(left || right).
/// Odd node count: last node duplicated.
pub struct MerkleTree {
    pub leaves: Vec<[u8; 32]>,
    pub levels: Vec<Vec<[u8; 32]>>,
}

impl MerkleTree {
    pub fn from_leaves(leaves: Vec<[u8; 32]>) -> Self {
        if leaves.is_empty() {
            return Self { leaves, levels: vec![] };
        }
        let mut levels = vec![leaves.clone()];
        let mut current = leaves.clone();
        while current.len() > 1 {
            let mut next = Vec::with_capacity((current.len() + 1) / 2);
            let mut i = 0;
            while i < current.len() {
                let left = current[i];
                let right = if i + 1 < current.len() { current[i+1] } else { current[i] };
                let mut buf = Vec::with_capacity(64);
                buf.extend_from_slice(&left);
                buf.extend_from_slice(&right);
                next.push(content_hash(&buf));
                i += 2;
            }
            levels.push(next.clone());
            current = next;
        }
        Self { leaves, levels }
    }

    pub fn root(&self) -> Option<[u8; 32]> {
        self.levels.last().and_then(|l| l.first().copied())
    }

    /// Proof for leaf index: list of (sibling_hash, is_left) from bottom up.
    pub fn proof(&self, mut idx: usize) -> Option<Vec<([u8;32], bool)>> {
        if idx >= self.leaves.len() { return None; }
        let mut proof = Vec::new();
        for level in &self.levels[..self.levels.len().saturating_sub(1)] {
            let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            let sibling = if sibling_idx < level.len() { level[sibling_idx] } else { level[idx] };
            let is_left = idx % 2 != 0; // sibling is left of node
            proof.push((sibling, is_left));
            idx /= 2;
        }
        Some(proof)
    }

    pub fn verify_proof(leaf: &[u8;32], proof: &[([u8;32], bool)], root: &[u8;32]) -> bool {
        let mut cur = *leaf;
        for (sib, is_left) in proof {
            let mut buf = Vec::with_capacity(64);
            if *is_left {
                buf.extend_from_slice(sib);
                buf.extend_from_slice(&cur);
            } else {
                buf.extend_from_slice(&cur);
                buf.extend_from_slice(sib);
            }
            cur = content_hash(&buf);
        }
        &cur == root
    }
}
