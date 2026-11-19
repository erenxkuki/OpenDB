//! ocf-decoder — partial and selective decoding (Phase 4).

#![deny(unsafe_code)]

pub mod partial;

pub use ocf_core::{CodecRegistry, OcfError, ResourceLimits, Result};
pub use partial::{DecodedSample, PartialDecoder};
pub use ocf_format::{Container, SampleEntry, SampleField};
pub use ocf_lossless::{Lz4Codec, ZstdCodec};
