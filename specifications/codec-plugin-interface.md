# Codec Plugin Interface (Draft)

**Status:** Phase 0 draft.

## 1. Goals

- Codecs are independently replaceable.
- No tight coupling between format and any single codec.
- Plugins use an explicit trusted extension interface; no arbitrary code execution from metadata.
- Codec identity is a stable u32 id registered in a codec registry.

## 2. Core traits (conceptual Rust)

```
pub trait Codec: Send + Sync {
    fn id(&self) -> CodecId;                 // u32
    fn name(&self) -> &'static str;
    fn version(&self) -> CodecVersion;        // (major, minor, patch)
    fn capabilities(&self) -> CodecCapabilities;

    fn compress(&self, input: &[u8], params: &CodecParams, out: &mut Vec<u8>) -> Result<CodecStats, CodecError>;
    fn decompress(&self, input: &[u8], out: &mut Vec<u8>, limits: &ResourceLimits) -> Result<CodecStats, CodecError>;
    fn decompress_range(&self, input: &[u8], range: Range<usize>, out: &mut Vec<u8>, limits: &ResourceLimits) -> Result<CodecStats, CodecError>;
}
```

`decompress_range` is optional (capability flag). Codecs that cannot do partial decode return `Unsupported` and the runtime falls back to full decode + slice.

## 3. Capabilities

```
struct CodecCapabilities {
    lossless: bool,                // must be true for default path
    supports_partial_decode: bool,
    supports_dictionary: bool,
    supports_streaming: bool,
    max_input_size: u64,
    max_output_size: u64,
}
```

## 4. Transform interface

Transforms are reversible preprocessing steps applied before/after codec.

```
pub trait Transform: Send + Sync {
    fn id(&self) -> TransformId;            // u32
    fn name(&self) -> &'static str;
    fn apply(&self, input: &[u8], params: &TransformParams, out: &mut Vec<u8>) -> Result<(), TransformError>;
    fn invert(&self, input: &[u8], params: &TransformParams, out: &mut Vec<u8>) -> Result<(), TransformError>;
    fn is_lossless(&self) -> bool;          // must be true for F0
}
```

A transform pipeline is an ordered list. Each step must be invertible for F0. Non-lossless transforms are only allowed in F2/F3 paths and must be recorded as semantic transformations.

## 5. Codec registry

- Registration at startup (static, compiled-in) or via trusted plugin (dynamic lib with explicit ABI version).
- Registry maps CodecId ↔ implementation. Unknown id → explicit error.
- Plugin loading requires signature/allow-list (security). No autoload from dataset metadata.
- Version recorded per chunk (`codec_id` + codec version in plan registry) for reproducibility.

## 6. Resource limits (passed to every decode)

```
struct ResourceLimits {
    max_decoded_bytes: u64,
    max_encoded_bytes: u64,
    max_allocations: u64,
    timeout_ms: u64,
    max_dependency_depth: u8,
}
```

Codecs MUST enforce these; violations → `ResourceLimitExceeded`.

## 7. Error taxonomy

- `UnsupportedCodec`
- `UnsupportedTransform`
- `CorruptData`
- `ChecksumMismatch`
- `ContentHashMismatch`
- `DependencyMissing`
- `DependencyCycle`
- `DepthExceeded`
- `ResourceLimitExceeded`
- `FidelityViolation`
- `VersionIncompatible`

All errors carry: codec/transform id, chunk id (if applicable), and a short message. No sample contents.

## 8. Compressed-domain operation registry

Each codec/representation declares supported operations (e.g. chunk lookup, metadata filter, hash verify, copy, column select, range extract, partial decode). Operation must specify input/output representation, required metadata, fidelity guarantee, complexity, whether decompression required. Unsupported ops return `UnsupportedOperation` with fallback hint. Never advertise unsupported ops.

## 9. Initial codec candidates

- Zstandard (general-purpose, high ratio, good speed).
- LZ4 (fast, lower ratio).
- (Later) additional codecs via registry.

Reversible transforms initially: delta encoding, bit packing, RLE, bitshuffle, dictionary encoding, byte shuffling, column separation, integer normalization (exact). Not all at once; establish interface + benchmark first.

## 10. Open questions

1. In-process trait vs C ABI plugin boundary.
2. Whether transforms should be composable into named "plans" stored once (yes, via transform plan registry) — adopted.
3. Parameter serialization format for plans (compact binary + canonical form for hashing).
