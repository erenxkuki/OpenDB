use ocf_core::{Codec, CodecCapabilities, CodecId, CodecParams, CodecStats, CodecVersion, OcfError, ResourceLimits, Result};

pub struct Lz4Codec {
    pub id: CodecId,
}

impl Lz4Codec {
    pub fn new(id: CodecId) -> Self { Self { id } }
}

impl Codec for Lz4Codec {
    fn id(&self) -> CodecId { self.id }
    fn name(&self) -> &'static str { "lz4" }
    fn version(&self) -> CodecVersion { (1, 9, 0) }
    fn capabilities(&self) -> CodecCapabilities {
        CodecCapabilities {
            lossless: true,
            supports_partial_decode: false,
            supports_dictionary: false,
            supports_streaming: true,
            max_input_size: u64::MAX,
            max_output_size: u64::MAX,
        }
    }

    fn compress(&self, input: &[u8], _params: &CodecParams, out: &mut Vec<u8>) -> Result<CodecStats> {
        let encoded = lz4::block::compress(input, None, false)
            .map_err(|e| OcfError::CorruptData(e.to_string()))?;
        out.extend_from_slice(&(input.len() as u64).to_le_bytes());
        out.extend_from_slice(&encoded);
        Ok(CodecStats {
            bytes_in: input.len() as u64,
            bytes_out: (8 + encoded.len()) as u64,
            ..Default::default()
        })
    }

    fn decompress(&self, input: &[u8], out: &mut Vec<u8>, limits: &ResourceLimits) -> Result<CodecStats> {
        if input.len() as u64 > limits.max_encoded_bytes {
            return Err(OcfError::ResourceLimitExceeded("encoded size".into()));
        }
        if input.len() < 8 {
            return Err(OcfError::CorruptData("truncated lz4 payload".into()));
        }
        let mut size_bytes = [0u8; 8];
        size_bytes.copy_from_slice(&input[..8]);
        let usize_expected = u64::from_le_bytes(size_bytes) as usize;
        if usize_expected as u64 > limits.max_decoded_bytes {
            return Err(OcfError::ResourceLimitExceeded("decoded size".into()));
        }
        let decoded = lz4::block::decompress(&input[8..], Some(usize_expected as i32))
            .map_err(|e| OcfError::CorruptData(e.to_string()))?;
        if decoded.len() != usize_expected {
            return Err(OcfError::CorruptData("lz4 size mismatch".into()));
        }
        out.extend_from_slice(&decoded);
        Ok(CodecStats {
            bytes_in: input.len() as u64,
            bytes_out: decoded.len() as u64,
            ..Default::default()
        })
    }
}
