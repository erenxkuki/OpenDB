use crate::plan::CompressionPlan;

/// Generate candidate plans given profile. Heuristic estimates.
pub fn generate_candidates(profile: &crate::profile::DatasetProfile) -> Vec<CompressionPlan> {
    let mut out = Vec::new();
    let mut id_counter: u128 = 1;

    // Codec 1 = zstd, 2 = lz4
    for &codec_id in &[1u32, 2u32] {
        for &level in &[1i32, 3, 9] {
            for &chunk_kb in &[64usize, 256, 1024] {
                let mut plan = CompressionPlan::new(id_counter, &format!("codec{}-lvl{}-{}kb", codec_id, level, chunk_kb));
                id_counter += 1;
                plan.codec_id = codec_id;
                plan.codec_level = level;
                plan.chunk_size = chunk_kb * 1024;
                // Heuristic estimates based on profile
                let ratio_base = if profile.entropy_bits_per_byte < 4.0 { 3.0 } else { 1.5 };
                let codec_factor = if codec_id == 1 { 1.2 } else { 1.0 };
                plan.estimated_ratio = ratio_base * codec_factor * (1.0 + profile.repetition_score);
                plan.estimated_encode_ms_per_mb = if codec_id == 1 { 20.0 * level as f64 } else { 5.0 };
                plan.estimated_decode_ms_per_mb = if codec_id == 1 { 8.0 } else { 2.0 };
                plan.estimated_random_access_penalty = (1024.0 / chunk_kb as f64).min(16.0) / 16.0;
                // Add delta transform for repetitive data
                if profile.repetition_score > 0.3 {
                    plan.transform_ids.push(1); // delta
                }
                out.push(plan);
            }
        }
    }
    out
}
