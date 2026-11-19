//! ocf-structural — content addressing, dedup, dictionaries, deltas, dependencies (Phase 3).

#![deny(unsafe_code)]

pub mod content_address;
pub mod dedup;
pub mod delta;
pub mod dependencies;
pub mod dictionary;
pub mod merkle;

pub use content_address::{ContentId, ContentStore, id_from_hash};
pub use dedup::{dedup_chunks, verify_dedup, DedupStats, DEP_EXACT_DUPLICATE};
pub use delta::{decode_delta, encode_delta, verify_delta_base, DEP_DELTA_BASE};
pub use dependencies::{validate_dependencies, MAX_DEPTH};
pub use dictionary::{Dictionary, DictionaryStore};
pub use merkle::MerkleTree;
