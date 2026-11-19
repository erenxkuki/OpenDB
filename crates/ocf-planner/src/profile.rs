use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetProfile {
    pub modality: String,
    pub total_bytes: u64,
    pub sample_count: u64,
    pub avg_sample_bytes: f64,
    pub entropy_bits_per_byte: f64,   // 0..8
    pub repetition_score: f64,         // 0..1
    pub distinct_byte_values: usize,
}

impl DatasetProfile {
    pub fn from_bytes(modality: &str, data: &[u8], sample_count: u64) -> Self {
        let mut freq = [0u64; 256];
        for &b in data { freq[b as usize] += 1; }
        let n = data.len().max(1) as f64;
        let mut entropy = 0.0;
        for &f in &freq {
            if f > 0 {
                let p = f as f64 / n;
                entropy -= p * p.log2();
            }
        }
        let distinct = freq.iter().filter(|&&f| f > 0).count();
        // repetition: fraction of bytes that equal previous byte (run-ish)
        let mut runs = 0u64;
        for i in 1..data.len() { if data[i] == data[i-1] { runs += 1; } }
        let repetition = if data.is_empty() { 0.0 } else { runs as f64 / data.len() as f64 };
        DatasetProfile {
            modality: modality.to_string(),
            total_bytes: data.len() as u64,
            sample_count,
            avg_sample_bytes: if sample_count == 0 { 0.0 } else { data.len() as f64 / sample_count as f64 },
            entropy_bits_per_byte: entropy,
            repetition_score: repetition,
            distinct_byte_values: distinct,
        }
    }
}
