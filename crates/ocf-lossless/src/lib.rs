//! ocf-lossless — codecs and reversible transforms (Phase 1).

#![deny(unsafe_code)]

pub mod lz4_codec;
pub mod transforms;
pub mod zstd_codec;

pub use lz4_codec::Lz4Codec;
pub use transforms::{DeltaTransform, PassthroughTransform, RleTransform};
pub use zstd_codec::ZstdCodec;

// Convenience re-exports of core traits.
pub use ocf_core::{Codec, CodecId, CodecParams, CodecStats, Transform, TransformId, TransformParams};