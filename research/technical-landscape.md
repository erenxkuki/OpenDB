# Technical Landscape — Compression & AI Data Infrastructure

**Status:** Phase 0 draft. Web research unavailable (no network in this environment); compiled from established knowledge. Claims should be re-verified against primary sources before finalizing.

## 1. General-purpose compression

### Zstandard (Zstd)
- Family: LZ77 + FSE (Finite State Entropy) + Huffman. Dictionary support. Streaming + seekable formats.
- Strengths: excellent ratio/speed trade-off, wide ecosystem, seekable format with frame index, dictionary training for small records.
- Weaknesses: not modality-aware; no native sample indexing; seekable format overhead.
- Reuse opportunity: use as default general-purpose codec in `ocf-lossless`. Leverage seekable format ideas for chunk-level random access. Use Zstd dictionary training for shared dictionaries (structural engine).
- Partial access: Zstd seekable format supports frame-level random access; within-frame still full frame decode. Our chunk granularity controls this.

### LZ4
- Family: LZ77, extremely fast.
- Strengths: very high decompression throughput, low CPU.
- Weaknesses: lower ratio than Zstd.
- Reuse: fast path for hot data, cache, low-latency random access. Complement to Zstd.

### OpenZL
- Newer (Meta, ~2024/2025): a composable compression framework that treats data as typed columns and applies a graph of codecs/transforms per column. Strong emphasis on structured/typed data and partial decoding.
- Strengths: column-aware, graph of transforms, typed inputs, partial decode, modern design targeting exactly this problem space.
- Weaknesses: newer, smaller ecosystem; less battle-tested than Zstd.
- Relation to OCF: OpenZL is the closest existing spiritual neighbor. OCF should study it closely (primary source review needed). Differentiation opportunities: OCF targets AI training lifecycle (planner, layouts, provenance, fidelity), not just compression. OCF must not claim novelty in column-aware compression; instead integrate and extend. Risk: if OpenZL covers 80% of needs, OCF should wrap/reuse rather than reinvent. Action item: obtain and read OpenZL source before Phase 1.

### Brotli, Snappy, Deflate, Gzip
- Brotli: good web text ratio, slower. Snappy: fast, low ratio. Mostly legacy for this use case. Not primary candidates but may be added via registry.

## 2. Columnar / AI data formats

### Apache Arrow
- In-memory columnar format; IPC/Feather. Excellent for typed arrays, zero-copy, interoperability.
- Reuse: Arrow-compatible interfaces for tensor/numeric data (section 15 of master prompt). OCF can store Arrow arrays compressed per-column. Arrow is a representation, not a storage/compression lifecycle; complement, not replacement.

### Lance
- Columnar format optimized for ML: versioning, random access, fast scan, vector search integration, blob storage, schema evolution.
- Strengths: ML-oriented, versioned, random access, partial column read.
- Relation: Lance is a storage format competitor/neighbor. OCF's scope (compression fabric + planner + layouts + provenance) is narrower and deeper on compression. Potential integration: OCF could become a compression backend for Lance-like stores, or OCF containers could be consumed by Lance readers. Need primary-source review.

### Parquet
- Columnar, compressed, predicate pushdown, row groups. Good for analytics; random access per row group is coarse.
- Reuse: borrow row-group/chunking ideas; OCF chunks are analogous but training-aware and modality-aware.

### Mosaic Streaming (MosaicML)
- Data streaming library for training: streams samples from cloud storage with shuffling, resumption, deterministic ordering. Focus on delivery, not compression format.
- Relation: OCF should integrate with such loaders (Phase 6). OCF provides storage/compression; loader provides scheduling. Complementary.

### WebDataset / TFRecord / Petastorm
- WebDataset: tar shards of samples; simple, streamable, but poor random access and dedup.
- TFRecord: protobuf records, TF-specific.
- Petastorm: Parquet-based ML data access.
- OCF aims to improve on these with chunking, indexing, partial decode, dedup, provenance.

## 3. Hashing

### BLAKE3
- Fast, parallelizable, 256-bit (or truncated), tree-structured → supports chunked/incremental and Merkle-like structures.
- Strong candidate for content identity + Merkle root. Also has keyed mode for MAC (not a replacement for signatures).
- Alternative: SHA-256 (ubiquitous, slower), xxHash (fast but not cryptographic).
- Decision: BLAKE3 candidate; verify in Phase 1. Use for content identity; keep checksums (CRC32C) separate for speed.

## 4. Learned compression

- Research area: neural codecs (e.g., for images/audio), learned lossless (e.g., L3, NNCP, CMIX) with strong ratios but huge compute.
- Status: established that learned methods can beat classical on specific modalities but at high encode/decode cost and model dependency.
- OpenDB stance: research extension only (Phase 8). Not in default path without evidence of overall system benefit. Deterministic decoding + model versioning are hard requirements.

## 5. Multimodal compression

- Modalities already have mature codecs (JPEG/PNG/WebP/AVIF for images; FLAC/Opus for audio; H.264/HEVC/AV1 for video).
- General-purpose recompression of already-compressed media usually yields little or negative gain; sometimes harmful.
- OCF stance: preserve original media bytes; dedup content-addressed; separate metadata; optional lossless transforms only where beneficial; never transcode without explicit policy. Cross-modal structural sharing (frames, transcripts) is research (Research B).

## 6. Distributed delivery & caching

- Object storage (S3) range requests, NVMe local cache, prefetch, async I/O.
- Reuse: tokio async runtime; object_store crate (candidate) for S3; local filesystem cache with LRU/size limits.

## 7. Key integration opportunities (genuine, not reinvention)

1. Codec registry + transform plans as first-class (like OpenZL's graph, but OCF-owned and training-aware).
2. Chunk-level content addressing + dedup across modalities.
3. Training-aware planner with multi-objective cost including delivery penalty.
4. Partial decode + selective fields as core, not afterthought.
5. Provenance + reproducibility as first-class metadata.
6. Adaptive physical layouts with atomic publication.

## 8. What already exists (do not reinvent)

- General compression (Zstd/LZ4) — use.
- Content hashing (BLAKE3) — use.
- Columnar in-memory (Arrow) — interoperate.
- Async runtime (tokio) — use.
- Object storage clients — use.

## 9. Gaps / open research

- Automated reversible transform pipeline discovery for AI datasets (Research A).
- Cross-modal structural compression (Research B).
- Fine-grained partial decode savings measurement (Research C).
- Adaptive layout evolution without downtime (Research D).
- Compression+scheduling co-optimization (Research E).
- Learned lossless practicality (Research F).

## 10. Action items before Phase 1

1. Obtain and read OpenZL source + docs; write a comparison note.
2. Obtain Lance spec; note overlap.
3. Benchmark Zstd vs LZ4 on representative text/image/tensor data for chunk sizes.
4. Decide BLAKE3 vs SHA-256 after benchmarking.
5. Decide Arrow version and FFI boundary.
