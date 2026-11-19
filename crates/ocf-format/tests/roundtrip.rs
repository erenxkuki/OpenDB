use ocf_format::{Chunk, Container};
use std::path::PathBuf;

#[test]
fn create_close_reopen_retrieve() {
    let mut c = Container::new("test-dataset");
    c.created_at_unix_ms = 12345;
    for i in 0..5u32 {
        let payload = vec![i as u8; 100 + i as usize];
        c.add_chunk(Chunk {
            chunk_id: (i as u128) + 1,
            modality: 0,
            fidelity: 0,
            sample_count: 10,
            raw_size: payload.len() as u64 * 2,
            codec_id: 1,
            transform_plan_index: 0,
            payload: payload.clone(),
            content_hash: [i as u8; 32],
            dependency_id: 0,
            dependency_kind: 0,
        });
    }
    let mut tmp = std::env::temp_dir();
    tmp.push("odcf_test.odcf");
    c.write_file(&tmp).unwrap();
    let c2 = Container::read_file(&tmp).unwrap();
    c2.verify_basic().unwrap();
    assert_eq!(c2.dataset_name, "test-dataset");
    assert_eq!(c2.chunks.len(), 5);
    for (a, b) in c.chunks.iter().zip(c2.chunks.iter()) {
        assert_eq!(a.chunk_id, b.chunk_id);
        assert_eq!(a.payload, b.payload);
        assert_eq!(a.content_hash, b.content_hash);
    }
    let _ = std::fs::remove_file(&tmp);
}
