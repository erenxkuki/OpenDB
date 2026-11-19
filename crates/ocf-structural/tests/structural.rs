use ocf_core::content_hash;
use ocf_format::Chunk;
use ocf_structural::*;

fn make_chunk(id: u128, payload: Vec<u8>) -> Chunk {
    Chunk {
        chunk_id: id,
        modality: 0,
        fidelity: 0,
        sample_count: 1,
        raw_size: payload.len() as u64,
        codec_id: 1,
        transform_plan_index: 0,
        content_hash: content_hash(&payload),
        payload,
        dependency_id: 0,
        dependency_kind: 0,
    }
}

#[test]
fn content_store_dedups() {
    let mut s = ContentStore::new();
    let a = s.put(b"hello");
    let b = s.put(b"hello");
    assert_eq!(a, b);
    assert_eq!(s.len(), 1);
    assert_eq!(s.get(&a), Some(b"hello".as_slice()));
}

#[test]
fn exact_dedup_saves_bytes() {
    let payload = vec![42u8; 100];
    let chunks = vec![
        make_chunk(1, payload.clone()),
        make_chunk(2, payload.clone()),
        make_chunk(3, payload.clone()),
    ];
    let total_before: u64 = chunks.iter().map(|c| c.payload.len() as u64).sum();
    let (out, stats) = dedup_chunks(chunks);
    assert_eq!(stats.duplicates_removed, 2);
    assert!(stats.bytes_saved > 0);
    assert_eq!(out.len(), 3);
    // canonical keeps payload, two refs empty
    let nonempty = out.iter().filter(|c| !c.payload.is_empty()).count();
    assert_eq!(nonempty, 1);
    let empty = out.iter().filter(|c| c.payload.is_empty()).count();
    assert_eq!(empty, 2);
    verify_dedup(&out).unwrap();
    // logical content hashes unchanged
    for c in &out {
        assert_eq!(c.content_hash, content_hash(&vec![42u8;100]));
    }
    let _ = total_before;
}

#[test]
fn delta_roundtrip() {
    let base = vec![10u8, 20, 30, 40];
    let target = vec![12u8, 18, 35, 40];
    let d = encode_delta(&base, &target).unwrap();
    let rec = decode_delta(&base, &d).unwrap();
    assert_eq!(rec, target);
    // mismatched lengths error
    assert!(encode_delta(&base, &[1,2]).is_err());
}

#[test]
fn dependency_validation_ok_and_cycle() {
    // linear chain: chunk1 -> chunk0
    let p0 = vec![1u8,2,3];
    let p1 = vec![4u8,5,6];
    let mut c0 = make_chunk(1, p0.clone());
    let mut c1 = make_chunk(2, p1.clone());
    // make c1 a reference to c0 via exact dup kind? use dependency_kind=2 (delta) with same length not needed; just existence
    c1.dependency_id = id_from_hash(&c0.content_hash);
    c1.dependency_kind = DEP_DELTA_BASE;
    // ensure lengths match for delta? not required for validation
    let chunks = vec![c0, c1];
    assert!(validate_dependencies(&chunks).is_ok());

    // cycle: two chunks referencing each other
    let mut a = make_chunk(10, vec![1u8]);
    let mut b = make_chunk(11, vec![2u8]);
    a.dependency_id = id_from_hash(&b.content_hash);
    a.dependency_kind = DEP_EXACT_DUPLICATE;
    b.dependency_id = id_from_hash(&a.content_hash);
    b.dependency_kind = DEP_EXACT_DUPLICATE;
    let chunks = vec![a, b];
    assert!(validate_dependencies(&chunks).is_err());
}

#[test]
fn missing_dependency_errors() {
    let mut c = make_chunk(1, vec![1u8,2]);
    c.dependency_id = 9999;
    c.dependency_kind = DEP_EXACT_DUPLICATE;
    assert!(validate_dependencies(&[c]).is_err());
}
