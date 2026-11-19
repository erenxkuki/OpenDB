use ocf_core::{content_hash, Codec};
use ocf_decoder::{PartialDecoder, SampleEntry, SampleField};
use ocf_format::Chunk;
use ocf_lossless::{Lz4Codec, ZstdCodec};
use std::sync::Arc;

fn zstd_chunk(id: u128, payload_src: &[u8]) -> Chunk {
    let mut enc = Vec::new();
    ZstdCodec::new(1).compress(payload_src, &ocf_core::CodecParams::default(), &mut enc).unwrap();
    Chunk {
        chunk_id: id, modality: 0, fidelity: 0, sample_count: 1,
        raw_size: payload_src.len() as u64, codec_id: 1, transform_plan_index: 0,
        payload: enc, content_hash: content_hash(payload_src),
        dependency_id: 0, dependency_kind: 0,
    }
}

#[test]
fn decode_chunk_roundtrip() {
    let data = b"hello world, this is a test payload".to_vec();
    let mut c = ocf_format::Container::new("d");
    c.add_chunk(zstd_chunk(1, &data));
    let reg = PartialDecoder::default_registry();
    let limits = ocf_core::ResourceLimits::default();
    let dec = PartialDecoder::decode_chunk(&c, 0, &reg, &limits).unwrap();
    assert_eq!(dec, data);
}

#[test]
fn decode_range() {
    let data: Vec<u8> = (0..100).collect();
    let mut c = ocf_format::Container::new("d");
    c.add_chunk(zstd_chunk(1, &data));
    let reg = PartialDecoder::default_registry();
    let limits = ocf_core::ResourceLimits::default();
    let r = PartialDecoder::decode_range(&c, 0, 10..20, &reg, &limits).unwrap();
    assert_eq!(r, (10..20).collect::<Vec<u8>>());
}

#[test]
fn exact_dup_decode() {
    use ocf_structural::{dedup_chunks, DEP_EXACT_DUPLICATE};
    let data = vec![7u8; 50];
    let chunks = vec![zstd_chunk(1, &data), zstd_chunk(2, &data)];
    let (deduped, _) = dedup_chunks(chunks);
    let mut c = ocf_format::Container::new("d");
    for ch in deduped { c.add_chunk(ch); }
    let reg = PartialDecoder::default_registry();
    let limits = ocf_core::ResourceLimits::default();
    // both chunks should decode to same data
    let d0 = PartialDecoder::decode_chunk(&c, 0, &reg, &limits).unwrap();
    let d1 = PartialDecoder::decode_chunk(&c, 1, &reg, &limits).unwrap();
    assert_eq!(d0, data);
    assert_eq!(d1, data);
}

#[test]
fn get_sample_fields() {
    let data: Vec<u8> = (0..64).collect();
    let mut c = ocf_format::Container::new("d");
    c.add_chunk(zstd_chunk(1, &data));
    // sample 1 references chunk 0, field 0 bytes [5..15]
    c.add_sample(SampleEntry {
        sample_id: 1,
        primary_chunk_index: 0,
        fields: vec![SampleField { field_id: 10, chunk_index: 0, byte_offset: 5, byte_length: 10 }],
    });
    let reg = PartialDecoder::default_registry();
    let limits = ocf_core::ResourceLimits::default();
    let s = PartialDecoder::get_sample(&c, 1, &reg, &limits).unwrap();
    assert_eq!(s.sample_id, 1);
    assert_eq!(s.fields.len(), 1);
    assert_eq!(s.fields[0].1, (5..15).collect::<Vec<u8>>());
}

#[test]
fn async_decode() {
    let data = b"async payload".to_vec();
    let mut c = ocf_format::Container::new("d");
    c.add_chunk(zstd_chunk(1, &data));
    let reg = Arc::new(PartialDecoder::default_registry());
    let limits = ocf_core::ResourceLimits::default();
    let carc = Arc::new(c);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let out = rt.block_on(PartialDecoder::decode_chunk_async(carc, 0, reg, limits)).unwrap();
    assert_eq!(out, data);
}
