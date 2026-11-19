use ocf_core::OcfError;

pub const DEP_DELTA_BASE: u8 = 2;

/// Reversible byte-wise delta: delta[i] = target[i] - base[i] (wrapping).
/// Requires equal lengths.
pub fn encode_delta(base: &[u8], target: &[u8]) -> Result<Vec<u8>, OcfError> {
    if base.len() != target.len() {
        return Err(OcfError::InvalidArgument("delta length mismatch".into()));
    }
    let mut out = Vec::with_capacity(target.len());
    for (b, t) in base.iter().zip(target.iter()) {
        out.push(t.wrapping_sub(*b));
    }
    Ok(out)
}

pub fn decode_delta(base: &[u8], delta: &[u8]) -> Result<Vec<u8>, OcfError> {
    if base.len() != delta.len() {
        return Err(OcfError::InvalidArgument("delta length mismatch".into()));
    }
    let mut out = Vec::with_capacity(delta.len());
    for (b, d) in base.iter().zip(delta.iter()) {
        out.push(b.wrapping_add(*d));
    }
    Ok(out)
}

/// Verify that base content hash matches expected.
pub fn verify_delta_base(base_content_hash: &[u8;32], expected: &[u8;32]) -> Result<(), OcfError> {
    if base_content_hash != expected {
        return Err(OcfError::ContentHashMismatch);
    }
    Ok(())
}
