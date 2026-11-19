use ocf_core::{OcfError, Result, Transform, TransformId, TransformParams};

/// Delta encoding on bytes: out[0]=in[0]; out[i]=in[i]-in[i-1] (wrapping).
pub struct DeltaTransform { pub id: TransformId }
impl Transform for DeltaTransform {
    fn id(&self) -> TransformId { self.id }
    fn name(&self) -> &'static str { "delta" }
    fn apply(&self, input: &[u8], _params: &TransformParams, out: &mut Vec<u8>) -> Result<()> {
        out.reserve(input.len());
        let mut prev: u8 = 0;
        for (i, &b) in input.iter().enumerate() {
            if i == 0 { out.push(b); prev = b; }
            else { out.push(b.wrapping_sub(prev)); prev = b; }
        }
        Ok(())
    }
    fn invert(&self, input: &[u8], _params: &TransformParams, out: &mut Vec<u8>) -> Result<()> {
        out.reserve(input.len());
        let mut prev: u8 = 0;
        for (i, &b) in input.iter().enumerate() {
            if i == 0 { out.push(b); prev = b; }
            else { let v = b.wrapping_add(prev); out.push(v); prev = v; }
        }
        Ok(())
    }
    fn is_lossless(&self) -> bool { true }
}

/// Simple run-length encoding: sequence of (value: u8, count: u8). Count 0 means 256.
pub struct RleTransform { pub id: TransformId }
impl Transform for RleTransform {
    fn id(&self) -> TransformId { self.id }
    fn name(&self) -> &'static str { "rle" }
    fn apply(&self, input: &[u8], _params: &TransformParams, out: &mut Vec<u8>) -> Result<()> {
        if input.is_empty() { return Ok(()); }
        let mut i = 0;
        while i < input.len() {
            let v = input[i];
            let mut run = 1usize;
            while i + run < input.len() && input[i+run] == v && run < 256 { run += 1; }
            let mut remaining = run;
            while remaining > 0 {
                let take = remaining.min(256);
                out.push(v);
                out.push(if take == 256 { 0 } else { take as u8 });
                remaining -= take;
            }
            i += run;
        }
        Ok(())
    }
    fn invert(&self, input: &[u8], _params: &TransformParams, out: &mut Vec<u8>) -> Result<()> {
        if input.len() % 2 != 0 {
            return Err(OcfError::CorruptData("rle odd length".into()));
        }
        for chunk in input.chunks(2) {
            let v = chunk[0];
            let c = if chunk[1] == 0 { 256usize } else { chunk[1] as usize };
            for _ in 0..c { out.push(v); }
        }
        Ok(())
    }
    fn is_lossless(&self) -> bool { true }
}

/// Passthrough (identity) transform.
pub struct PassthroughTransform { pub id: TransformId }
impl Transform for PassthroughTransform {
    fn id(&self) -> TransformId { self.id }
    fn name(&self) -> &'static str { "passthrough" }
    fn apply(&self, input: &[u8], _params: &TransformParams, out: &mut Vec<u8>) -> Result<()> {
        out.extend_from_slice(input); Ok(())
    }
    fn invert(&self, input: &[u8], _params: &TransformParams, out: &mut Vec<u8>) -> Result<()> {
        out.extend_from_slice(input); Ok(())
    }
    fn is_lossless(&self) -> bool { true }
}
