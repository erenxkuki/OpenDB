//! ocf-runtime — async I/O, prefetch, cache, scheduling, metrics (Phase 6 partial).

#![deny(unsafe_code)]

pub mod cache;
pub mod metrics;
pub mod prefetch;

pub use cache::Cache;
pub use metrics::RuntimeMetrics;
pub use prefetch::Prefetcher;
