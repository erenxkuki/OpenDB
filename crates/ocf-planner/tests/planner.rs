use ocf_planner::{DatasetProfile, HardConstraints, Planner};

#[test]
fn selects_plan() {
    let data = b"aaaaabbbbbccccc".repeat(1000);
    let profile = DatasetProfile::from_bytes("text", &data, 100);
    let p = Planner::new();
    let plan = p.plan(&profile).expect("plan");
    assert!(plan.estimated_ratio > 1.0);
    assert!(plan.chunk_size > 0);
}

#[test]
fn ranked_plans_sorted() {
    let data = vec![0u8; 100_000];
    let profile = DatasetProfile::from_bytes("tensor", &data, 10);
    let p = Planner::new();
    let ranked = p.plan_all_ranked(&profile);
    assert!(!ranked.is_empty());
    for w in ranked.windows(2) {
        assert!(w[0].1 <= w[1].1 + 1e-9);
    }
}

#[test]
fn hard_constraint_filters() {
    let data = vec![0u8; 1000];
    let profile = DatasetProfile::from_bytes("text", &data, 10);
    // very strict decode latency -> filters out high-level zstd
    let mut hc = HardConstraints::default();
    hc.max_decode_ms_per_mb = 1.0;
    let p = Planner::with_constraints(hc);
    let plan = p.plan(&profile);
    // lz4 should be chosen (lower decode)
    if let Some(pl) = plan {
        assert!(pl.estimated_decode_ms_per_mb <= 2.5);
    }
}

#[test]
fn plan_persist_roundtrip() {
    use std::path::PathBuf;
    let data = b"hello".repeat(100);
    let profile = DatasetProfile::from_bytes("text", &data, 10);
    let p = Planner::new();
    let plan = p.plan(&profile).unwrap();
    let mut tmp = std::env::temp_dir();
    tmp.push("plan_test.json");
    ocf_planner::save_plan(&plan, &tmp).unwrap();
    let loaded = ocf_planner::load_plan(&tmp).unwrap();
    assert_eq!(loaded.plan_id, plan.plan_id);
    assert_eq!(loaded.codec_id, plan.codec_id);
    let _ = std::fs::remove_file(&tmp);
}
