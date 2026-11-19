use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalLayout {
    pub layout_id: u128,
    pub name: String,
    pub chunk_order: Vec<usize>,
    pub created_at_unix_ms: u64,
}

impl PhysicalLayout {
    pub fn new(layout_id: u128, name: &str, chunk_order: Vec<usize>) -> Self {
        Self { layout_id, name: name.to_string(), chunk_order, created_at_unix_ms: 0 }
    }

    /// Verify this layout is a valid permutation of 0..n.
    pub fn is_valid_permutation(&self, n: usize) -> bool {
        if self.chunk_order.len() != n { return false; }
        let mut seen = vec![false; n];
        for &i in &self.chunk_order {
            if i >= n || seen[i] { return false; }
            seen[i] = true;
        }
        true
    }
}
