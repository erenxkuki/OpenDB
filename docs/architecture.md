# OCF Architecture — Open Compression Fabric

**Status:** Phase 0 draft. Subject to review.
**Scope:** Compression subsystem of OpenDB only.

## 1. Purpose and non-goals

OCF is the compression subsystem of OpenDB. It owns representation, transformation, compression, structural sharing, semantic/reduced-fidelity representations, partial decoding, integrity verification, and delivery of compressed data.

OCF is **not**:
- a generic ZIP replacement,
- a thin wrapper around Zstandard,
- a codec benchmark harness alone.

OCF **is** a foundational data-engine component that must compose with ingestion, versioning, provenance, indexing, training planning, caching, and distributed retrieval.

## 2. Top-level decomposition

OCF is split into independently testable subsystems. Each maps to a crate.

| Crate | Responsibility |
|---|---|
| `ocf-core` | Compression lifecycle, codec/transform registries, fidelity policy, plan execution, errors, resource limits, version compatibility. |
| `ocf-format` | ODCF container: header, manifest, chunk, directory, indexes. Pure format logic, no codec dependency. |
| `ocf-lossless` | General-purpose codecs (Zstd, LZ4) + reversible transforms (delta, bit-pack, RLE, bitshuffle, dict, byte-shuffle, column separation, integer normalization). |
| `ocf-structural` | Content-addressed storage, exact dedup, shared dictionaries, reversible deltas, dependency management. |
| `ocf-semantic` | Reduced-fidelity / semantic representations, fidelity validation. Always separate from lossless path. |
| `ocf-planner` | Dataset profiling, candidate plan generation, cost model, plan selection, plan persistence. |
| `ocf-decoder` | Partial/selective decoding, integrity verification, bounded decode memory. |
| `ocf-runtime` | Async I/O, prefetch, cache, scheduling, metrics. |
| `ocf-bench` | Benchmark harness (OCF Bench). |

Dependency direction (must be acyclic): `ocf-format` and `ocf-core` are lowest. `ocf-lossless`, `ocf-structural`, `ocf-semantic` depend on core+format. `ocf-planner`, `ocf-decoder` depend on those. `ocf-runtime` depends on decoder. `ocf-bench` depends on all. `opendb-python` (PyO3) is built separately and depends on runtime.

`ocf-format` must remain independent of higher-level training orchestration. This is a hard constraint.

## 3. Core architectural principles (constraints)

1. **Logical/physical separation.** A logical dataset is independent of physical representation. Multiple physical layouts may exist for one logical dataset/version. Changing physical representation must not silently change logical data.
2. **Lossless-first.** F0 (bit-exact) is the default. For F0: `Decode(Encode(x)) == x` byte-for-byte. Lossy/reduced-fidelity (F2/F3) are explicit, separately versioned, opt-in. Never silently replace original with reduced-fidelity. Keep references to original.
3. **Independent decoding.** Prefer independently decodable chunks with bounded dependencies. No single enormous compressed stream for a whole dataset.
4. **Reproducibility.** Every compressed object carries or references immutable metadata sufficient to decode. No undocumented external state.
5. **Modularity.** Codecs, transforms, indexing, backends independently replaceable. No tight coupling to one codec.
6. **Performance transparency.** Every optimization measurable: ratio, encode/decode time, bytes read/decoded, cache behavior, runtime stats.
7. **Backward compatibility.** Old datasets remain readable when new strategies are introduced. Never silently reinterpret old metadata.

## 4. Fidelity classes

- **F0** — bit-exact reconstruction. Default.
- **F1** — exact logical sample reconstruction through a documented reversible representation.
- **F2** — numerically bounded reconstruction (requires explicit authorization).
- **F3** — semantic / reduced-fidelity representation (requires explicit authorization).

F2/F3 require: transformation identity + version, parameters, input content hash, output content hash, fidelity class, expected reconstruction characteristics, validation results. Original data references maintained whenever possible.

## 5. Data flow

### Ingestion → encode
1. Ingestion produces logical samples with schema + modality.
2. Dataset profiler (`ocf-planner`) computes entropy, repetition, sizes, access hints.
3. Compression planner generates candidate plans, evaluates under hard constraints + multi-objective cost, selects one.
4. Structural engine dedups content-addressed objects, builds shared dictionaries/deltas.
5. Lossless/semantic engines encode chunks per plan.
6. `ocf-format` writes ODCF container with manifest, chunk directory, sample index, dictionaries, provenance, integrity metadata.
7. Atomic publication of dataset version.

### Read → decode
1. Reader opens dataset (fidelity, access mode).
2. Sample/chunk index resolves logical request → chunk set.
3. Partial decoder fetches only needed chunks/ranges/fields.
4. Integrity verifier checks content hashes + checksums.
5. Returns decoded data with bounded memory.

### Planning feedback
Training runtime emits telemetry (GPU idle, batch prep latency, throughput, CPU, decode queue, cache hit, network, worker imbalance). Planner may adapt prefetch, worker allocation, cache placement, chunk scheduling, physical layout selection. Sampling policy, seed, ordering, partitioning stay separately defined and reproducible — never silently altered.

## 6. Compression planner cost model

Multi-objective, not ratio-only:

`J(P) = w_s·S(P) + w_c·C(P) + w_d·D(P) + w_r·R(P) + w_t·T(P)`

- S = normalized storage consumption
- C = compression CPU cost
- D = decompression cost
- R = random-access penalty
- T = expected training data delivery penalty
- weights workload-specific.

Hard constraints (reject regardless of ratio): max decode latency, max memory, min throughput, required fidelity, max dependency depth.

Two execution modes:
- **Offline planning:** explore candidates, benchmark, estimate, produce reusable plan.
- **Online execution:** execute approved plan, bounded adaptations, monitor, no expensive unrestricted search.

Selected plan identity + version stored with each chunk.

## 7. Structural compression

- Content-addressed, immutable objects keyed by stable hash (BLAKE3 candidate).
- Exact dedup only initially; near-duplicate detection is advisory (similarity ≠ interchangeability).
- Shared dictionaries, metadata dedup, reversible deltas, shared binary components.
- Dependency safety: verify identity/hash/availability, max depth, no cycles, no delete of referenced objects.
- Garbage collection must never delete referenced objects.

## 8. Partial decompression

Capabilities: chunk-level, sample-level, range, column-level, modality-specific, selective, async, parallel, cached. Avoid decoding whole dataset for small subsets. Independent compressed streams per field/modality. Distinguish clearly: partial storage retrieval vs partial decompression vs selective field decoding vs compressed-domain ops. Only claim compressed-domain ops when mathematically valid for the representation; provide fallback.

## 9. Adaptive physical layouts

Multiple physical layouts per logical dataset/version (e.g. sequential text training, multimodal training, random access, archival). All reference same logical version. Workload observation → candidate layout → benchmark → verify → compare → publish atomically → keep previous → GC when unreferenced. Migration must not interrupt active jobs. Never expose partially constructed layout.

## 10. Metadata and provenance (layers)

- Dataset: ID, version, schema, modality defs, creation time, source, fidelity policy.
- Chunk: ID, offset, encoded/raw size, codec ID, transform plan ID, dict ID, content hash.
- Sample: ID, modality, logical location, content hash, fields, parent relationships.
- Lineage: source, transformations, params, parent/derived versions, creation process, validation results.

Provenance kept independent of compression-specific metadata.

## 11. Integrity

- Lossless round-trip F0 across empty/tiny/large/random/repetitive/binary/numeric/multimodal.
- Stable content hash (BLAKE3 candidate); distinguish content identity vs corruption detection vs authenticity (checksum ≠ crypto auth).
- Merkle dataset structures: chunk membership proofs, dataset root identity, version comparison, canonical serialization.
- Dependency safety checks.
- Deterministic reconstruction: all params + codec versions recorded.
- Crash safety: interruption during compress/write/manifest/index/migration/publication must never yield a valid-looking incomplete dataset.

## 12. Security / resource safety

Treat compressed datasets as untrusted. Limits: max decoded/encoded chunk size, max metadata/dict size, max dependency depth, max allocation, overflow protection, offset validation, bounds checking, decoder timeouts/cancellation, resource accounting. Decompression-bomb prevention. Fuzz: header/manifest/index/codec dispatch/chunk decode/dependency resolution. No arbitrary code from metadata. Plugins via trusted extension interface only.

## 13. Module boundaries & invariants (review checklist)

- `ocf-format` has no dependency on codecs or training orchestration.
- No circular dependencies.
- No hidden global mutable state.
- No unbounded background tasks.
- Every public module documents purpose/inputs/outputs/invariants/failure modes/perf/compat.

## 14. Open questions (unresolved decisions)

1. Exact ODCF header size and alignment (see spec).
2. Chunking defaults per modality (fixed bytes vs fixed samples vs variable) — needs benchmarks.
3. Whether sample index lives inside container or sidecar for very large datasets.
4. Codec registry ABI: in-process Rust trait vs dynamic plugin.
5. SQLite vs custom metadata store for local.
6. Merkle tree arity and canonical serialization details.
7. Learned compression: deferred to Phase 8; no default-path inclusion without evidence.

These are recorded, not resolved, per Phase 0 requirements.
