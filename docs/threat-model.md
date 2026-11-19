# Security Threat Model — OCF (Draft)

**Status:** Phase 0 draft.

## 1. Trust boundary

Compressed datasets are **untrusted input**. They may come from external sources, be corrupted in transit, or be maliciously crafted. The engine must never trust container metadata, chunk data, or dependency references without validation.

## 2. Threats

| Threat | Impact | Mitigation |
|---|---|---|
| Decompression bomb (small input → huge output) | Memory exhaustion, DoS | max_decoded_bytes, max_allocations, resource accounting, streaming decode with abort |
| Malformed header/manifest/index | Crash, OOB read | strict parsing, bounds checks, CRC, reject unknown required features |
| Integer overflow in size fields | OOB, huge alloc | checked arithmetic, reject oversized |
| Path traversal via metadata strings | File overwrite | treat metadata as data; no path interpretation; sandboxed I/O |
| Dependency cycle / deep chain | Stack overflow, infinite loop | depth limit (<=8), cycle detection |
| Missing dependency → silent fallback | Wrong data | explicit error, no fallback |
| Codec confusion (wrong codec_id) | Misdecode, garbage | registry lookup; unknown → error |
| Plugin code execution | RCE | trusted plugin interface only; allow-list; signature; no autoload from metadata |
| Hash collision (content identity) | Wrong dedup merge | use 256-bit (BLAKE3) for identity; u128 only where verified against full hash |
| Provenance tampering | Wrong lineage | content-hash provenance; Merkle root |
| Crash mid-write | Partial dataset mistaken as valid | atomic publication, CRC, publication marker |
| Telemetry leaking sample contents | Privacy | no sample contents in metrics by default |

## 3. Resource limits (enforced)

- max decoded chunk size
- max encoded chunk size
- max metadata size
- max dictionary size
- max dependency depth (<=8)
- max allocation count/bytes
- integer-overflow protection
- offset validation and bounds checking
- decoder timeouts / cancellation
- explicit resource accounting

## 4. Fuzz targets

- header parsing
- manifest parsing
- index parsing
- codec dispatch
- chunk decoding
- dependency resolution
- transform pipeline invert

## 5. Unsafe code

- `#![deny(unsafe_code)]` at crate roots in Phase 0.
- If unsafe becomes necessary (SIMD, FFI), document invariants, minimize scope, add tests/sanitizers.

## 6. Open questions

1. Whether to run decoders in separate processes for isolation (heavy; defer).
2. Plugin signing scheme.
3. Rate limiting for remote fetches.
