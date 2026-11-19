use crate::content_address::id_from_hash;
use ocf_core::OcfError;
use ocf_format::Chunk;

pub const DEP_EXACT_DUPLICATE: u8 = 1;

#[derive(Debug, Default)]
pub struct DedupStats {
    pub input_chunks: usize,
    pub output_chunks: usize,
    pub duplicates_removed: usize,
    pub bytes_saved: u64,
}

/// Exact deduplication: chunks with identical content_hash are collapsed.
/// The first occurrence keeps its payload; subsequent ones become references
/// (empty payload, dependency_id pointing at the canonical chunk).
pub fn dedup_chunks(chunks: Vec<Chunk>) -> (Vec<Chunk>, DedupStats) {
    let input_chunks = chunks.len();
    let mut seen: std::collections::HashMap<[u8;32], usize> = std::collections::HashMap::new();
    let mut out: Vec<Chunk> = Vec::with_capacity(chunks.len());
    let mut duplicates = 0usize;
    let mut bytes_saved = 0u64;

    for mut c in chunks {
        if let Some(&canon_idx) = seen.get(&c.content_hash) {
            // reference
            let canon_id = out[canon_idx].chunk_id; // not used for id
            let _ = canon_id;
            bytes_saved += c.payload.len() as u64;
            c.payload.clear();
            c.dependency_id = id_from_hash(&c.content_hash); // will be resolved to canonical's hash id
            // better: point to canonical's content hash id
            c.dependency_id = id_from_hash(&out[canon_idx].content_hash);
            c.dependency_kind = DEP_EXACT_DUPLICATE;
            duplicates += 1;
        } else {
            seen.insert(c.content_hash, out.len());
        }
        out.push(c);
    }

    // Fix: dependency_id should point to canonical content hash, already set above.
    let stats = DedupStats {
        input_chunks,
        output_chunks: out.len(),
        duplicates_removed: duplicates,
        bytes_saved,
    };
    (out, stats)
}

/// Verify that a reference chunk's dependency resolves to an existing canonical chunk
/// with matching content hash.
pub fn verify_dedup(chunks: &[Chunk]) -> Result<(), OcfError> {
    // map id -> canonical index (chunk with non-empty payload for that content hash)
    let mut by_id: std::collections::HashMap<u128, usize> = std::collections::HashMap::new();
    for (i, c) in chunks.iter().enumerate() {
        if !c.payload.is_empty() {
            by_id.insert(id_from_hash(&c.content_hash), i);
        }
    }
    for c in chunks {
        if c.dependency_kind == DEP_EXACT_DUPLICATE {
            match by_id.get(&c.dependency_id) {
                Some(&idx) => {
                    if chunks[idx].content_hash != c.content_hash {
                        return Err(OcfError::ContentHashMismatch);
                    }
                    if chunks[idx].payload.is_empty() {
                        return Err(OcfError::DependencyMissing("canonical payload missing".into()));
                    }
                }
                None => return Err(OcfError::DependencyMissing("exact dup base".into())),
            }
        }
    }
    Ok(())
}
