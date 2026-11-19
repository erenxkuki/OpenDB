use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default)]
pub struct RuntimeMetrics {
    pub samples_served: AtomicU64,
    pub bytes_decoded: AtomicU64,
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
}

impl RuntimeMetrics {
    pub fn new() -> Self { Self::default() }
    pub fn record_sample(&self) { self.samples_served.fetch_add(1, Ordering::Relaxed); }
    pub fn record_bytes(&self, n: u64) { self.bytes_decoded.fetch_add(n, Ordering::Relaxed); }
    pub fn record_cache_hit(&self) { self.cache_hits.fetch_add(1, Ordering::Relaxed); }
    pub fn record_cache_miss(&self) { self.cache_misses.fetch_add(1, Ordering::Relaxed); }
    pub fn snapshot(&self) -> (u64,u64,u64,u64) {
        (self.samples_served.load(Ordering::Relaxed),
         self.bytes_decoded.load(Ordering::Relaxed),
         self.cache_hits.load(Ordering::Relaxed),
         self.cache_misses.load(Ordering::Relaxed))
    }
}
