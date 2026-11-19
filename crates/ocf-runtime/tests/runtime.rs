use ocf_runtime::{Cache, Prefetcher, RuntimeMetrics};

#[test]
fn cache_evicts() {
    let mut c = Cache::new(10);
    c.insert(1, vec![1u8; 6]);
    c.insert(2, vec![2u8; 6]);
    // first should be evicted
    assert!(c.get(1).is_none());
    assert!(c.get(2).is_some());
    assert!(c.used_bytes() <= 10);
}

#[test]
fn prefetch_queue() {
    let mut p = Prefetcher::new(2);
    p.enqueue(5);
    p.enqueue(6);
    p.enqueue(7); // dropped
    assert_eq!(p.len(), 2);
    assert_eq!(p.dequeue(), Some(5));
    assert_eq!(p.dequeue(), Some(6));
    assert!(p.is_empty());
}

#[test]
fn metrics_snapshot() {
    let m = RuntimeMetrics::new();
    m.record_sample();
    m.record_bytes(42);
    m.record_cache_hit();
    let (s,b,h,miss) = m.snapshot();
    assert_eq!(s,1); assert_eq!(b,42); assert_eq!(h,1); assert_eq!(miss,0);
}
