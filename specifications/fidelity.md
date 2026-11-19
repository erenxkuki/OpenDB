# Fidelity and Correctness Specification (Draft)

**Status:** Phase 0 draft.

## 1. Fidelity classes

- **F0** — bit-exact reconstruction. Default for all data.
- **F1** — exact logical sample reconstruction through a documented reversible representation (e.g., canonicalized text encoding). Bytes may differ but logical sample is identical and reversible.
- **F2** — numerically bounded reconstruction. Requires explicit authorization. Error bounds must be declared and validated.
- **F3** — semantic / reduced-fidelity representation. Requires explicit authorization. May lose information irreversibly.

## 2. Hard rules

1. F0 is the default everywhere.
2. F2/F3 require explicit opt-in at dataset creation and at read time.
3. Never silently replace original data with a reduced-fidelity representation.
4. Maintain references to original data whenever possible (provenance).
5. Fidelity policy ceiling at dataset level; per-chunk fidelity must be <= policy.
6. A chunk's fidelity is recorded in its descriptor and verified on read against requested fidelity.

## 3. Required metadata per non-F0 transformation

For F1/F2/F3 transforms record:
- transformation identity (id + version)
- parameters
- input content hash
- output content hash
- fidelity class
- expected reconstruction characteristics (e.g., error bounds for F2)
- validation results

## 4. Correctness invariants

### 4.1 Lossless round-trip (F0)
For all F0 objects: `Decode(Encode(x)) == x` byte-for-byte.

Test matrices:
- empty, tiny, large, random, highly repetitive, binary, numeric arrays, multimodal samples.
- boundary sizes (0, 1, block-size-1, block-size, block-size+1, 2^k, 2^k-1).
- all-zero, all-0xFF, alternating.

### 4.2 Content identity
Stable content hash (BLAKE3 candidate). `hash(x)` deterministic across runs/machines. Hash of canonical metadata reproducible.

### 4.3 Dependency safety
- Dictionary identity verified (hash).
- Delta-base identity verified.
- Dependency availability checked.
- Max depth enforced (<=8).
- No cycles.
- Missing dependency → explicit error.

### 4.4 Deterministic reconstruction
Recorded dataset must not depend on undocumented runtime config. Codec versions + transform params + plan id stored.

### 4.5 Crash safety
Interruption during compress/write/manifest/index/migration/publication must never produce a valid-looking incomplete dataset. Verified by fault-injection tests.

## 5. Validation procedures

- On encode: round-trip check in debug/test builds; hash computed and stored.
- On decode: recompute content hash and compare; checksum verification; fidelity class check against request.
- On dataset open: validate header/footer CRC, manifest, chunk directory counts, index consistency (sample_count matches), Merkle root if present.

## 6. Fuzz targets

- Header parsing.
- Manifest parsing.
- Index parsing.
- Codec dispatch.
- Chunk decoding.
- Dependency resolution.
- Transform pipeline invert.

## 7. Open questions

1. Whether F1 canonicalization rules are defined per-modality or global.
2. Error-bound representation for F2 (absolute vs relative, per-channel).
3. Whether to store original bytes alongside F3 representations (recommended yes when feasible).
