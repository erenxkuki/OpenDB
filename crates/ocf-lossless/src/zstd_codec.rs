use ocf_core::{Codec, CodecCapabilities, CodecId, CodecParams, CodecStats, CodecVersion, OcfError, ResourceLimits, Result};

pub struct ZstdCodec {
    pub id: CodecId,
}

impl ZstdCodec {
    pub fn new(id: CodecId) -> Self { Self { id } }
}

impl Codec for ZstdCodec {
    fn id(&self) -> CodecId { self.id }
    fn name(&self) -> &'static str { "zstd" }
    fn version(&self) -> CodecVersion { (1, 5, 0) }
    fn capabilities(&self) -> CodecCapabilities {
        CodecCapabilities {
            lossless: true,
            supports_partial_decode: false,
            supports_dictionary: true,
            supports_streaming: true,
            max_input_size: u64::MAX,
            max_output_size: u64::MAX,
        }
    }

    fn compress(&self, input: &[u8], params: &CodecParams, out: &mut Vec<u8>) -> Result<CodecStats> {
        let level = if params.level == 0 { 3 } else { params.level };
        let encoded = zstd::encode_all(input, level)
            .map_err(|e| OcfError::CorruptData(e.to_string()))?;
        out.extend_from_slice(&encoded);
        Ok(CodecStats {
            bytes_in: input.len() as u64,
            bytes_out: encoded.len() as u64,
            ..Default::default()
        })
    }

    fn decompress(&self, input: &[u8], out: &mut Vec<u8>, limits: &ResourceLimits) -> Result<CodecStats> {
        if input.len() as u64 > limits.max_encoded_bytes {
            return Err(OcfError::ResourceLimitExceeded("encoded size".into()));
        }
        let decoded = zstd::decode_all(input)
            .map_err(|e| OcfError::CorruptData(e.to_string()))?;
        if decoded.len() as u64 > limits.max_decoded_bytes {
            return Err(OcfError::ResourceLimitExceeded("decoded size".into()));
        }
        out.extend_from_slice(&decoded);
        Ok(CodecStats {
            bytes_in: input.len() as u64,
            bytes_out: decoded.len() as u64,
            ..Default::default()
        })
    }
}
