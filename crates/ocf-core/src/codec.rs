use crate::{ResourceLimits, Result};
use std::ops::Range;

pub type CodecId = u32;
pub type CodecVersion = (u16, u16, u16);

#[derive(Debug, Clone, Copy)]
pub struct CodecCapabilities {
    pub lossless: bool,
    pub supports_partial_decode: bool,
    pub supports_dictionary: bool,
    pub supports_streaming: bool,
    pub max_input_size: u64,
    pub max_output_size: u64,
}

#[derive(Debug, Clone, Default)]
pub struct CodecParams {
    pub level: i32,
    pub dictionary: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CodecStats {
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub encode_time_ns: u64,
    pub decode_time_ns: u64,
}

pub trait Codec: Send + Sync {
    fn id(&self) -> CodecId;
    fn name(&self) -> &'static str;
    fn version(&self) -> CodecVersion;
    fn capabilities(&self) -> CodecCapabilities;

    fn compress(&self, input: &[u8], params: &CodecParams, out: &mut Vec<u8>) -> Result<CodecStats>;
    fn decompress(&self, input: &[u8], out: &mut Vec<u8>, limits: &ResourceLimits) -> Result<CodecStats>;

    fn decompress_range(&self, input: &[u8], range: Range<usize>, out: &mut Vec<u8>, limits: &ResourceLimits) -> Result<CodecStats> {
        let mut buf = Vec::new();
        self.decompress(input, &mut buf, limits)?;
        let start = range.start.min(buf.len());
        let end = range.end.min(buf.len());
        out.extend_from_slice(&buf[start..end]);
        Ok(CodecStats { bytes_in: input.len() as u64, bytes_out: (end - start) as u64, ..Default::default() })
    }
}
