<div align="center">

# OpenDB

### Open Compression Fabric (OCF)

**An intelligent, verifiable, training-aware data engine for AI training infrastructure.**

[![Status](https://img.shields.io/badge/status-Phase%207%20complete-brightgreen)](#status)
[![Language](https://img.shields.io/badge/language-Rust-orange)](#)
[![Python](https://img.shields.io/badge/python-PyO3-blue)](#)
[![License](https://img.shields.io/badge/license-Apache--2.0%20%7C%20MIT-lightgrey)](#license)
[![Tests](https://img.shields.io/badge/tests-30%2B%20passing-success)](#)

> **Store once logically. Represent intelligently. Retrieve selectively. Decode minimally. Verify everything.**

</div>

---

## What is OpenDB?

OpenDB is not a generic database and not a generic file compressor. It is a specialized, high-performance engine for **storing, indexing, validating, and delivering very large AI training datasets** with minimal unnecessary storage, transfer, and decoding overhead.

Compression is a foundational subsystem, not an optional feature.

## Core promise

| Principle | Meaning |
|---|---|
| 🧠 Intelligent | The engine understands the dataset *and* the workload, then picks the right representation. |
| 🔒 Verifiable | Every byte is content-addressed, hashed, and round-trip verified. |
| ⚡ Training-aware | Optimizes end-to-end training throughput, not just compression ratio. |
| 📦 Partial by default | Decode only what is needed — chunk, sample, field, or range. |
| 🔁 Reproducible | Immutable metadata; no undocumented external state. |

## Architecture at a glance

```
┌─────────────────────────────────────────────────────────────┐
│                      OpenDB / OCF                           │
├─────────────┬──────────────┬──────────────┬─────────────────┤
│  ocf-core   │ ocf-format   │ ocf-lossless │ ocf-structural  │
│  traits,    │ ODCF v0.1    │ Zstd, LZ4,   │ content addr,   │
│  registries │ container    │ transforms   │ dedup, deltas   │
├─────────────┼──────────────┼──────────────┼─────────────────┤
│ ocf-decoder │ ocf-planner  │ ocf-runtime  │  ocf-layout     │
│ partial     │ cost model,  │ cache,       │ adaptive        │
│ decode      │ plan select  │ prefetch     │ layouts         │
└─────────────┴──────────────┴──────────────┴─────────────────┘
```

## Repository layout

```
opendb/
├── Cargo.toml
├── crates/          # Rust workspace (10 crates)
├── docs/            # Architecture, threat model, roadmap
├── specifications/  # ODCF format, codec & fidelity specs
├── research/        # Technical landscape, hypotheses
├── benchmarks/      # Methodology
├── examples/
└── tests/
```

## Status

| Phase | Name | Status |
|---|---|---|
| 0 | Research & Architecture | ✅ Complete |
| 1 | OCF Foundation (codecs, transforms, fidelity) | ✅ Complete |
| 2 | ODCF Prototype (container format) | ✅ Complete |
| 3 | Structural Compression (dedup, deltas) | ✅ Complete |
| 4 | Partial Decompression | ✅ Complete |
| 5 | Intelligent Planner | ✅ Complete |
| 6 | Training Runtime (partial — no PyTorch in env) | 🟡 Partial |
| 7 | Adaptive Physical Layouts | ✅ Complete |
| 8 | Research Extensions | 📄 Documented only |

**Tests:** 30+ passing · `cargo test --workspace` green.

## Quick start

```bash
# Build the workspace
cargo build --workspace

# Run all tests
cargo test --workspace
```

## Documentation index

- [`docs/architecture.md`](docs/architecture.md) — OCF architecture & module boundaries
- [`specifications/odcf-v0.1.md`](specifications/odcf-v0.1.md) — ODCF binary format spec
- [`specifications/codec-plugin-interface.md`](specifications/codec-plugin-interface.md) — codec plugin contract
- [`specifications/fidelity.md`](specifications/fidelity.md) — fidelity classes & correctness
- [`research/technical-landscape.md`](research/technical-landscape.md) — comparison of existing systems
- [`research/research-hypotheses.md`](research/research-hypotheses.md) — research directions
- [`docs/threat-model.md`](docs/threat-model.md) — security & resource safety
- [`docs/roadmap.md`](docs/roadmap.md) — development roadmap
- [`benchmarks/methodology.md`](benchmarks/methodology.md) — benchmarking methodology

## Design principles

1. **Logical/physical separation** — multiple physical layouts per logical dataset.
2. **Lossless-first** — F0 (bit-exact) is the default; reduced fidelity is explicit & opt-in.
3. **Independent decoding** — chunk-granular, bounded dependencies.
4. **Reproducibility** — immutable metadata, no undocumented state.
5. **Modularity** — codecs, transforms, indexes, backends are swappable.
6. **Performance transparency** — everything is measurable.
7. **Backward compatibility** — old datasets stay readable.

## License

Apache-2.0 OR MIT (pending final decision).
