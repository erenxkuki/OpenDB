use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionPlan {
    pub plan_id: u128,
    pub name: String,
    pub codec_id: u32,
    pub codec_level: i32,
    pub transform_ids: Vec<u32>,
    pub chunk_size: usize,
    pub fidelity: u8, // FidelityClass as u8
    pub estimated_ratio: f64,
    pub estimated_encode_ms_per_mb: f64,
    pub estimated_decode_ms_per_mb: f64,
    pub estimated_random_access_penalty: f64,
}

impl CompressionPlan {
    pub fn new(plan_id: u128, name: &str) -> Self {
        Self {
            plan_id, name: name.to_string(), codec_id: 1, codec_level: 3,
            transform_ids: Vec::new(), chunk_size: 1 << 20,
            fidelity: 0, estimated_ratio: 1.0,
            estimated_encode_ms_per_mb: 10.0,
            estimated_decode_ms_per_mb: 5.0,
            estimated_random_access_penalty: 0.0,
        }
    }
}
