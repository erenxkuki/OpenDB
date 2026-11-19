/// Content hashing utilities for OpenDB.
///
/// Uses BLAKE3 (256-bit) for content identity. This is the canonical
/// content hash used throughout OCF for content addressing and dedup.

pub fn content_hash(bytes: &[u8]) -> [u8; 32] {
    *blake3::hash(bytes).as_bytes()
}

pub fn content_hash_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}
