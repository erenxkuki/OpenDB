# Roadmap

## Completed

- **Phase 0** — Research & architecture: tech landscape, ODCF v0.1 spec, threat model, benchmark methodology.
- **Phase 1** — OCF foundation: codec/transform traits, registries, Zstd + LZ4, Delta/RLE transforms, fidelity, resource limits. (5 tests)
- **Phase 2** — ODCF prototype: 128-byte header, manifest, 120-byte chunk descriptors, sample index, payload/footer, file read/write. (1 test)
- **Phase 3** — Structural compression: BLAKE3 content addressing, exact dedup, dependency validation, reversible delta, dictionary store, Merkle tree. (7 tests)
- **Phase 4** — Partial decompression: chunk/range/sample decoding, reference resolution, bounded memory, async wrapper. (5 tests)
- **Phase 5** — Intelligent planner: profiling, candidate generation, multi-objective cost model, hard constraints, ranked selection, plan persistence. (4 tests)
- **Phase 6 (partial)** — Runtime: LRU cache, prefetcher, metrics. PyTorch/JAX integration deferred (no ML framework in env). (3 tests)
- **Phase 7** — Adaptive layouts: observer, candidate generation, atomic publish/rollback, retirement. (3 tests)

## Totals

- 10 crates · 30+ tests · `cargo test --workspace` green.

## Deferred / research only

- Phase 6 ML framework integration.
- Phase 8: learned lossless compression, GPU-assisted decoding, distributed compression, compressed-domain compute — documented in `research/research-hypotheses.md`, not implemented (integrate only with evidence).

## Open questions

See `specifications/odcf-v0.1.md` §14 and `docs/architecture.md`.
