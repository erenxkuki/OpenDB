use ocf_core::{CodecRegistry, OcfError, ResourceLimits, Result};
use ocf_format::Container;
use ocf_structural::{decode_delta, DEP_DELTA_BASE, DEP_EXACT_DUPLICATE, id_from_hash};

#[derive(Debug)]
pub struct DecodedSample {
    pub sample_id: u64,
    pub fields: Vec<(u32, Vec<u8>)>,
}

pub struct PartialDecoder;

impl PartialDecoder {
    /// Find canonical chunk index for an exact-duplicate reference.
    fn canonical_index(container: &Container, chunk_idx: usize) -> Option<usize> {
        let c = &container.chunks[chunk_idx];
        container.chunks.iter().enumerate().find_map(|(i, other)| {
            if i != chunk_idx && other.content_hash == c.content_hash && !other.payload.is_empty() {
                Some(i)
            } else { None }
        })
    }

    /// Resolve base chunk index for a delta reference.
    fn base_index(container: &Container, chunk_idx: usize) -> Option<usize> {
        let c = &container.chunks[chunk_idx];
        container.chunks.iter().enumerate().find_map(|(i, other)| {
            if id_from_hash(&other.content_hash) == c.dependency_id { Some(i) } else { None }
        })
    }

    pub fn decode_chunk(
        container: &Container,
        chunk_idx: usize,
        registry: &CodecRegistry,
        limits: &ResourceLimits,
    ) -> Result<Vec<u8>> {
        if chunk_idx >= container.chunks.len() {
            return Err(OcfError::InvalidArgument("chunk index out of range".into()));
        }
        let chunk = &container.chunks[chunk_idx];

        // Exact duplicate reference -> decode canonical.
        if chunk.dependency_kind == DEP_EXACT_DUPLICATE {
            if let Some(ci) = Self::canonical_index(container, chunk_idx) {
                return Self::decode_chunk(container, ci, registry, limits);
            }
            return Err(OcfError::DependencyMissing("exact dup canonical not found".into()));
        }

        // Delta reference -> decode base + apply delta.
        if chunk.dependency_kind == DEP_DELTA_BASE {
            if let Some(bi) = Self::base_index(container, chunk_idx) {
                let base_decoded = Self::decode_chunk(container, bi, registry, limits)?;
                let target = decode_delta(&base_decoded, &chunk.payload)?;
                if target.len() as u64 > limits.max_decoded_bytes {
                    return Err(OcfError::ResourceLimitExceeded("decoded size".into()));
                }
                return Ok(target);
            }
            return Err(OcfError::DependencyMissing("delta base not found".into()));
        }

        // Normal chunk.
        let codec = registry.get(chunk.codec_id)
            .ok_or(OcfError::UnsupportedCodec(chunk.codec_id))?;
        let mut out = Vec::new();
        codec.decompress(&chunk.payload, &mut out, limits)?;
        Ok(out)
    }

    pub fn decode_range(
        container: &Container,
        chunk_idx: usize,
        range: std::ops::Range<usize>,
        registry: &CodecRegistry,
        limits: &ResourceLimits,
    ) -> Result<Vec<u8>> {
        let full = Self::decode_chunk(container, chunk_idx, registry, limits)?;
        let start = range.start.min(full.len());
        let end = range.end.min(full.len());
        Ok(full[start..end].to_vec())
    }

    pub fn get_sample(
        container: &Container,
        sample_id: u64,
        registry: &CodecRegistry,
        limits: &ResourceLimits,
    ) -> Result<DecodedSample> {
        let entry = container.samples.iter().find(|s| s.sample_id == sample_id)
            .ok_or_else(|| OcfError::InvalidArgument("sample not found".into()))?;
        let mut fields = Vec::new();
        for f in &entry.fields {
            let decoded = Self::decode_chunk(container, f.chunk_index as usize, registry, limits)?;
            let off = f.byte_offset as usize;
            let ln = f.byte_length as usize;
            let end = (off + ln).min(decoded.len());
            let start = off.min(decoded.len());
            fields.push((f.field_id, decoded[start..end].to_vec()));
        }
        Ok(DecodedSample { sample_id: entry.sample_id, fields })
    }

    pub fn default_registry() -> CodecRegistry {
        let r = CodecRegistry::new();
        r.register(std::sync::Arc::new(ocf_lossless::ZstdCodec::new(1)));
        r.register(std::sync::Arc::new(ocf_lossless::Lz4Codec::new(2)));
        r
    }

    // Async wrappers
    pub async fn decode_chunk_async(
        container: std::sync::Arc<Container>,
        chunk_idx: usize,
        registry: std::sync::Arc<CodecRegistry>,
        limits: ResourceLimits,
    ) -> Result<Vec<u8>> {
        tokio::task::spawn_blocking(move || {
            Self::decode_chunk(&container, chunk_idx, &registry, &limits)
        }).await.map_err(|e| OcfError::InvalidArgument(e.to_string()))?
    }
}
