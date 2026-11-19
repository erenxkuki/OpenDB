use ocf_core::{Codec, CodecParams, ResourceLimits, Transform, TransformParams};
use ocf_lossless::{DeltaTransform, Lz4Codec, PassthroughTransform, RleTransform, ZstdCodec};

fn roundtrip_codec<C: Codec>(c: &C, data: &[u8]) {
    let mut enc = Vec::new();
    c.compress(data, &CodecParams::default(), &mut enc).unwrap();
    let mut dec = Vec::new();
    c.decompress(&enc, &mut dec, &ResourceLimits::default()).unwrap();
    assert_eq!(dec, data, "codec {} roundtrip failed", c.name());
}

#[test]
fn zstd_roundtrip_matrix() {
    let c = ZstdCodec::new(1);
    let cases: Vec<Vec<u8>> = vec![
        vec![],
        vec![42],
        vec![0u8; 1000],
        (0..256).map(|x| x as u8).collect(),
        (0..100_000).map(|i| (i * 37 % 251) as u8).collect(),
        b"the quick brown fox jumps over the lazy dog ".repeat(500),
    ];
    for d in &cases { roundtrip_codec(&c, d); }
}

#[test]
fn lz4_roundtrip_matrix() {
    let c = Lz4Codec::new(2);
    let cases: Vec<Vec<u8>> = vec![
        vec![],
        vec![7],
        vec![0u8; 1000],
        (0..256).map(|x| x as u8).collect(),
        (0..100_000).map(|i| (i * 37 % 251) as u8).collect(),
        b"aaaaabbbbbccccc".repeat(1000),
    ];
    for d in &cases { roundtrip_codec(&c, d); }
}

fn roundtrip_transform<T: Transform>(t: &T, data: &[u8]) {
    let mut a = Vec::new();
    t.apply(data, &TransformParams::default(), &mut a).unwrap();
    let mut b = Vec::new();
    t.invert(&a, &TransformParams::default(), &mut b).unwrap();
    assert_eq!(b, data, "transform {} roundtrip failed", t.name());
}

#[test]
fn transform_roundtrip_matrix() {
    let d = DeltaTransform { id: 1 };
    let r = RleTransform { id: 2 };
    let p = PassthroughTransform { id: 3 };
    let cases: Vec<Vec<u8>> = vec![
        vec![],
        vec![5],
        vec![0u8; 500],
        (0..=255).collect(),
        b"aaabbbccc".to_vec(),
    ];
    for c in &cases {
        roundtrip_transform(&d, c);
        roundtrip_transform(&r, c);
        roundtrip_transform(&p, c);
    }
}

#[test]
fn resource_limit_rejected() {
    let c = ZstdCodec::new(1);
    let data = vec![0u8; 1024];
    let mut enc = Vec::new();
    c.compress(&data, &CodecParams::default(), &mut enc).unwrap();
    let tiny = ResourceLimits { max_decoded_bytes: 1, ..ResourceLimits::default() };
    let mut dec = Vec::new();
    let res = c.decompress(&enc, &mut dec, &tiny);
    assert!(res.is_err());
}

#[test]
fn registry_roundtrip() {
    use ocf_core::{CodecRegistry, TransformRegistry};
    use std::sync::Arc;
    let cr = CodecRegistry::new();
    cr.register(Arc::new(ZstdCodec::new(1)));
    cr.register(Arc::new(Lz4Codec::new(2)));
    assert!(cr.contains(1) && cr.contains(2));
    assert_eq!(cr.len(), 2);
    let tr = TransformRegistry::new();
    tr.register(Arc::new(DeltaTransform { id: 10 }));
    assert!(tr.contains(10));
}
