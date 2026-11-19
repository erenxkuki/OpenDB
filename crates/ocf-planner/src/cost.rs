use crate::plan::CompressionPlan;

#[derive(Debug, Clone)]
pub struct CostWeights {
    pub w_storage: f64,
    pub w_encode: f64,
    pub w_decode: f64,
    pub w_random_access: f64,
    pub w_delivery: f64,
}

impl Default for CostWeights {
    fn default() -> Self {
        Self { w_storage: 0.4, w_encode: 0.15, w_decode: 0.2, w_random_access: 0.15, w_delivery: 0.1 }
    }
}

#[derive(Debug, Clone)]
pub struct HardConstraints {
    pub max_decode_ms_per_mb: f64,
    pub max_memory_bytes: u64,
    pub min_throughput_mb_s: f64,
    pub required_fidelity: u8,
    pub max_dependency_depth: u8,
}

impl Default for HardConstraints {
    fn default() -> Self {
        Self { max_decode_ms_per_mb: 100.0, max_memory_bytes: 256*1024*1024,
               min_throughput_mb_s: 10.0, required_fidelity: 0, max_dependency_depth: 8 }
    }
}

pub struct CostModel { pub weights: CostWeights, pub constraints: HardConstraints }

impl CostModel {
    pub fn feasible(&self, plan: &CompressionPlan) -> bool {
        if plan.fidelity > self.constraints.required_fidelity { return false; } // stored fidelity must be <= required (lower better)
        if plan.estimated_decode_ms_per_mb > self.constraints.max_decode_ms_per_mb { return false; }
        // memory/throughput approximated: skip exact for prototype
        true
    }

    /// J(P) lower is better.
    pub fn cost(&self, plan: &CompressionPlan) -> f64 {
        let s = 1.0 / plan.estimated_ratio.max(0.01); // storage (lower better -> higher ratio lower cost)
        let c = plan.estimated_encode_ms_per_mb / 100.0;
        let d = plan.estimated_decode_ms_per_mb / 100.0;
        let r = plan.estimated_random_access_penalty;
        let t = d; // delivery approximated by decode
        self.weights.w_storage * s
            + self.weights.w_encode * c
            + self.weights.w_decode * d
            + self.weights.w_random_access * r
            + self.weights.w_delivery * t
    }
}
