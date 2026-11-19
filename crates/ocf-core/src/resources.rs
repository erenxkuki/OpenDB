#[derive(Debug, Clone, Copy)]
pub struct ResourceLimits {
    pub max_decoded_bytes: u64,
    pub max_encoded_bytes: u64,
    pub max_allocations: u64,
    pub timeout_ms: u64,
    pub max_dependency_depth: u8,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_decoded_bytes: 256 * 1024 * 1024,
            max_encoded_bytes: 256 * 1024 * 1024,
            max_allocations: 1 << 20,
            timeout_ms: 30_000,
            max_dependency_depth: 8,
        }
    }
}
