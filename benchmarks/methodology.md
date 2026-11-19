# OCF Bench — Benchmarking Methodology (Draft)

**Status:** Phase 0 draft. No results yet. Never fabricate numbers.

## 1. Principles

- Fairness: keep constant dataset, hardware, logical sample sequence, batch size, training framework, preprocessing, storage location, cache conditions.
- Run cold-cache and warm-cache; sequential and random; single-node and distributed.
- Publish scripts, configs, raw results, methodology. Report failures and limitations.

## 2. Comparators (study, don't assume equivalence)

- Zstandard (raw)
- LZ4 (raw)
- Apache Arrow-based representations
- Lance
- Mosaic Streaming
- (Others as relevant)

Describe capability differences accurately; not all provide identical features.

## 3. Dataset categories

- Text corpora.
- Image-text collections.
- Audio datasets.
- Video datasets.
- Numerical tensors.
- Structured metadata.
- Mixed-modality datasets.

Record characteristics and licensing.

## 4. Metrics

### Compression
- original bytes, encoded bytes, ratio
- compression throughput, decompression throughput
- CPU time, memory consumption, startup cost

### Retrieval
- sequential throughput
- random-access latency (p50/p95/p99)
- partial retrieval overhead
- unnecessary bytes fetched/decoded

### Training
- samples/sec
- batch prep latency
- GPU idle time
- worker imbalance
- storage/network bandwidth
- CPU utilization, peak memory

## 5. Method

1. Fix dataset + version.
2. Define workload (access pattern, batch size, workers, seed).
3. Run each system with identical workload.
4. Repeat N times; report median + variance.
5. Separate cold/warm.
6. Verify integrity of all outputs.

## 6. Anti-patterns (avoid)

- Comparing different datasets.
- Cherry-picking best run.
- Ignoring decode cost.
- Claiming superiority without reproducible measurement.
- Reporting ratios without throughput.
