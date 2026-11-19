# ODCF v0.1 — OpenDB Compressed Data Format Specification (Draft)

**Status:** Phase 0 draft. Not yet implemented. Subject to review.
**Format name:** ODCF (OpenDB Compressed Data Format)
**Container scope:** single dataset version (logical dataset + one physical representation).

## 0. Design goals

1. Independently decodable chunks.
2. Multiple codecs/transforms in one dataset.
3. Sample-level indexing and partial retrieval.
4. Content addressing and structural sharing.
5. Fidelity metadata per chunk.
6. Integrity verification (hashes + checksums + optional Merkle).
7. Version evolution and forward compatibility.
8. Compact metadata suitable for millions of chunks.
9. Crash-safe publication (atomic metadata update).

## 1. General rules

- **Endianness:** little-endian for all multi-byte integers.
- **Alignment:** 8-byte alignment for section starts where noted; payload may be unaligned but chunk payloads SHOULD be 8-byte aligned for SIMD-friendly codecs.
- **Integers:** fixed widths as specified; no variable-length integers in v0.1 (simplicity; revisit if metadata size dominates).
- **Magic:** ASCII `ODCF` (0x4F 0x44 0x43 0x46).
- **Unknown fields/extensions:** readers MUST ignore unknown optional sections and unknown trailing fields within a section if the section version permits; MUST reject unknown *required* features (feature flags).
- **Versioning:** `(major, minor)`. Major change = incompatible reader. Minor change = additive. v0.1 = (0,1). Readers accept same major, minor >= file's minor? No: readers accept files with major equal and minor <= reader's minor (reader must support older minor). Actually: reader supports file if file.major == reader.major and file.minor <= reader.minor. Forward-compatible within major.

## 2. Logical container layout

```
[File Header]            fixed size, at offset 0
[Manifest Section]       dataset-level metadata
[Schema Section]         schema + modality definitions
[Chunk Directory]        array of ChunkDescriptor (compact)
[Sample Index]           sample_id -> chunk range / field offsets
[Dictionary Directory]   shared dictionaries
[Transform Plan Registry] plan_id -> plan descriptor
[Compressed Payloads]    chunk payloads (concatenated, addressed by offset)
[Provenance Section]     lineage metadata
[Integrity Section]      hashes, Merkle root (optional)
[Footer]                 index locations, magic, version
```

Sections are located via offsets in the header/footer; not necessarily in this order on disk, but readers should not assume order beyond header-at-0 and footer-at-end. Writers SHOULD write in this order.

Each section has a small section header: `section_type: u32`, `section_version: u16`, `flags: u16`, `size: u64`, `crc32c: u32`, `reserved: u32`.

## 3. File header (fixed, 128 bytes)

| Offset | Field | Type | Notes |
|---|---|---|---|
| 0 | magic | [u8;4] | `ODCF` |
| 4 | format_major | u16 | 0 |
| 6 | format_minor | u16 | 1 |
| 8 | header_size | u32 | 128 |
| 12 | feature_flags | u64 | bitset of required/optional features |
| 20 | dataset_id | [u8;16] | UUID |
| 36 | dataset_version | u64 | monotonic |
| 44 | manifest_offset | u64 | |
| 52 | manifest_size | u64 | |
| 60 | chunk_dir_offset | u64 | |
| 68 | chunk_dir_size | u64 | |
| 76 | sample_index_offset | u64 | |
| 84 | sample_index_size | u64 | |
| 92 | dict_dir_offset | u64 | |
| 100 | dict_dir_size | u64 | |
| 108 | footer_offset | u64 | |
| 116 | header_crc32c | u32 | CRC over bytes 0..116 |
| 120 | reserved | [u8;8] | zero |

Feature flags bits (v0.1):
- bit 0: has_sample_index
- bit 1: has_dictionary_dir
- bit 2: has_transform_plan_registry
- bit 3: has_provenance
- bit 4: has_integrity_merkle
- bit 5: has_footer_index (footer carries section index)
- bits 6-63: reserved (must be 0 in v0.1; unknown required bits reject)

Rationale: fixed 128-byte header allows single-read header fetch and keeps offsets simple. 8-byte alignment of section offsets recommended.

## 4. Footer (fixed tail, 64 bytes)

Located at `footer_offset`. Contains a section index (offsets for all sections) for recovery when header corrupted, plus magic repeat.

| Field | Type |
|---|---|
| section_count | u32 |
| sections | array of (section_type:u32, offset:u64, size:u64) |
| footer_magic | [u8;4] = `ODCF` |
| format_major | u16 |
| format_minor | u16 |
| footer_crc32c | u32 |

Rationale: dual location (header + footer) aids recovery and crash detection.

## 5. Manifest section

Dataset-level metadata. Compact binary (not JSON) for production; JSON debug view supported.

Fields:
- `dataset_name`: length-prefixed UTF-8 (u32 len + bytes).
- `created_at_unix_ms`: u64
- `source_description`: length-prefixed UTF-8 (optional, empty allowed)
- `schema_id`: u128 (reference into Schema section)
- `fidelity_policy`: u8 (0 = F0-only default; 1 = allows F1; 2 = allows F2; 3 = allows F3) — policy ceiling, per-chunk fidelity must be <= policy.
- `default_codec_id`: u32
- `default_transform_plan_id`: u128
- `chunk_count`: u64
- `sample_count`: u64
- `raw_total_bytes`: u64
- `encoded_total_bytes`: u64 (payload bytes, excluding metadata)
- `content_hash_dataset`: [u8;32] (BLAKE3 over canonical manifest+chunks; see §9)
- `reserved`: 16 bytes.

## 6. Schema section

- `modality_count`: u32
- For each modality:
  - `modality_id`: u32
  - `name`: length-prefixed UTF-8
  - `kind`: u8 (0=text,1=image,2=audio,3=video,4=tensor,5=structured,6=other)
  - `field_count`: u32
  - For each field: `field_id: u32`, `name`, `dtype: u8`, `nullable: u8`, `modality_ref: u32`.
- `schema_version`: u32.

Dtype enum (v0.1): 0=u8,1=i8,2=u16,3=i16,4=u32,5=i32,6=u64,7=i64,8=f32,9=f64,10=bool,11=bytes,12=utf8,13=list,14=struct,255=unknown.

## 7. Chunk directory

Array of `ChunkDescriptor`. Count = manifest.chunk_count.

Compact fixed part (48 bytes) + variable part (content hash always 32 bytes; dict/dep optional via flags).

```
struct ChunkDescriptor {          // fixed 48 bytes
    chunk_id: u128,              // 16
    modality: u32,               // 4
    fidelity: u8,                // 1
    flags: u8,                   // 1: bit0 has_dict, bit1 has_dep, bit2 has_transform_override
    reserved0: u16,              // 2
    sample_count: u32,           // 4
    raw_size: u64,               // 8
    encoded_size: u64,           // 8
    codec_id: u32,               // 4
    transform_plan_id: u128?     // NO — see variable
}
```

Hmm, 128-bit transform_plan_id in every chunk is 16 bytes → header bloat for millions of chunks. Open question. For v0.1 we store `transform_plan_id: u32` (index into plan registry) instead of u128, and the registry holds the full u128. This keeps chunk descriptor compact. Decision recorded: use u32 plan index. (This is a v0.1 decision; revisit.)

Revised fixed part (48 bytes):
```
chunk_id: u128            (16)
modality: u32             (4)
fidelity: u8              (1)
flags: u8                 (1)
reserved0: u16            (2)
sample_count: u32         (4)
raw_size: u64             (8)
encoded_size: u64         (8)
codec_id: u32             (4)
transform_plan_index: u32 (4)
payload_offset: u64       (8)   // absolute file offset of payload
payload_size: u64         (8)   // = encoded_size usually; separate for alignment/padding
```
That's 16+4+1+1+2+4+8+8+4+4+8+8 = 68 bytes. Hmm, let me recount: 16+4=20, +1=21, +1=22, +2=24, +4=28, +8=36, +8=44, +4=48, +4=52, +8=60, +8=68. 68 bytes per chunk. For 10M chunks → 680MB of directory. That may be too much; consider chunk grouping or larger chunks. Open question: chunk count targets. We'll benchmark. For v0.1 keep simple.

Variable part after fixed:
- `content_hash: [u8;32]` always.
- if flags.has_dict: `dictionary_id: u128` (16).
- if flags.has_dep: `dependency_id: u128` (16) + `dependency_kind: u8` (1).
- if flags.has_transform_override: `transform_plan_id_override: u128` (16) (rare).

Checksum: each chunk payload has its own CRC32C stored? We store `chunk_crc32c: u32` at end of descriptor (add 4 bytes). Final: fixed 68 + 4 = 72 + variable.

## 8. Sample index

Maps logical sample → chunk + field offsets.

Structure:
- `sample_count: u64`
- `entry_stride` determined by modality/field count. To keep simple and generic, v0.1 uses a variable-length entry:
  - For each sample: `sample_id: u64`, `primary_chunk_index: u32`, `field_count: u16`, then for each field: `field_id: u32`, `chunk_index: u32`, `byte_offset_in_chunk: u64`, `byte_length: u64`.
- Sorted by sample_id for binary search. Optional hash index for O(1) lookup (memory cost) — deferred to Phase 4.

Rationale: sample index enables partial retrieval without decoding whole dataset. Size grows with sample count × fields; for huge datasets consider sparse/paged index (open question).

## 9. Integrity

### Content hashing
- Algorithm: BLAKE3 (candidate; final decision in Phase 1 after benchmark). 32-byte output.
- Chunk content hash = BLAKE3(chunk raw bytes) for F0 (identity of logical content). For encoded payload, separate `payload_hash = BLAKE3(encoded bytes)` for corruption detection.
- Dataset root = BLAKE3 over canonical serialization of manifest + sorted chunk descriptors + transform plan registry. Canonical serialization specified in §10.

### Checksums
- CRC32C per section and per chunk payload for fast corruption detection. Not cryptographic.

### Merkle (optional, feature flag)
- Binary Merkle tree over chunk content hashes. Root stored in integrity section.
- Provides chunk membership proofs.
- Tree arity and leaf hashing domain separation specified in a later revision; v0.1 leaves as optional/experimental.

## 10. Canonical serialization (for hashing)

To make content hashes reproducible:
- All strings UTF-8, length-prefixed u32.
- Integers little-endian fixed width.
- Arrays prefixed by u64 count.
- Maps sorted by key.
- No padding in canonical form.
- Floating point not used in canonical metadata (avoid NaN issues); use integer representations.

## 11. Dependency safety rules

- `dependency_id` references a chunk or dictionary by its content hash (u128 truncated? Use full 32 bytes? For compactness v0.1 uses u128 = first 16 bytes of BLAKE3; collision risk accepted for v0.1 with verification against full hash in dependency object). Open question: full hash in dependency ref costs 32 bytes. v0.1 uses u128 for compactness; revisit if needed.
- Max dependency depth: 8 (hard limit, configurable down).
- No circular references (validated at write and read).
- Missing dependency → explicit error, never silent fallback.
- GC must not delete referenced objects.

## 12. Crash safety & publication

- Writes use temp file + fsync + atomic rename for manifest/index.
- A dataset version is "published" only after footer + header CRC valid and a publication marker (in manifest or sidecar) is set.
- Readers reject versions without valid publication marker.
- Incomplete writes (missing footer, bad CRC) → treated as absent/corrupt, not as valid dataset.

## 13. Compatibility & migration

- Readers accept (major=0, minor<=1).
- New codecs added via registry; old chunks keep their codec_id. Reader must have codec or fail with explicit "unsupported codec" error.
- Format migration: provide a tool to rewrite a dataset to a newer format version, preserving logical content hashes.

## 14. Open questions (recorded, not resolved)

1. Chunk size defaults per modality and resulting chunk counts.
2. Sample index scalability (paged/sparse index).
3. Dependency reference hash width (u128 vs 256-bit).
4. Merkle arity/proof format.
5. Whether payloads should be in same file or sidecar (for object storage range requests).
6. Metadata compression (compress manifest/index with Zstd?) — likely yes, but adds complexity.
7. Encryption at rest — out of scope v0.1.

## 15. What v0.1 explicitly does NOT include

- Learned/semantic compression in default path.
- GPU decoding.
- Distributed storage backend specifics.
- Adaptive layout migration (Phase 7).
- Compressed-domain compute beyond metadata filtering.
