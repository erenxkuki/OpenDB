use crate::{OcfError, Result};

pub type TransformId = u32;

#[derive(Debug, Clone, Default)]
pub struct TransformParams {
    pub data: Vec<u8>,
}

pub trait Transform: Send + Sync {
    fn id(&self) -> TransformId;
    fn name(&self) -> &'static str;
    fn apply(&self, input: &[u8], params: &TransformParams, out: &mut Vec<u8>) -> Result<()>;
    fn invert(&self, input: &[u8], params: &TransformParams, out: &mut Vec<u8>) -> Result<()>;
    fn is_lossless(&self) -> bool;
}

// keep OcfError referenced
#[allow(dead_code)]
fn _touch(_e: OcfError) {}
