# Research Hypotheses — OCF

**Status:** Phase 0 draft. Hypotheses, not results. Each must be experimentally verified before integration.

## Framing

Clearly distinguish:
- **Established technology** — known techniques we reuse.
- **Integration opportunity** — combining existing pieces in a useful way.
- **Potential original contribution** — only if experiments support it.

Never claim novelty by renaming.

## Research A — Compression Plan Discovery

- Established: codec selection heuristics, transform pipelines, AutoML-style search.
- Gap: automated search over reversible transform pipelines specifically for AI training datasets, under training-aware cost model.
- Hypothesis: An offline search over a bounded set of reversible transforms + codecs, evaluated under a multi-objective cost (storage, decode, random-access, delivery), finds plans that outperform hand-chosen defaults on at least one workload class without violating fidelity.
- Method: define transform/codec candidate set; profile dataset; enumerate/beam-search plans; benchmark each; select by cost. Persist plan id+version.
- Metrics: storage bytes, encode/decode throughput, random-access p95, end-to-end samples/s, fidelity verification.
- Failure conditions: search cost exceeds benefit; no plan beats default by >X%; plans overfit to benchmark.
- Limitations: search space combinatorial; needs bounded candidate set; dataset-specific.

## Research B — Cross-Modal Structural Compression

- Established: content-addressed dedup, shared dictionaries, media codecs.
- Gap: sharing structural representations across modalities (e.g., video frames + extracted frames + transcript timestamps) while retaining independent access.
- Hypothesis: For datasets with derived modalities, content-addressing + reference edges reduce storage vs independent storage, with bounded access overhead.
- Method: store original + derived; link via provenance; dedup exact shared bytes; measure.
- Metrics: storage reduction, retrieval latency overhead, integrity.
- Failure: derived representations rarely byte-identical; sharing gains small; complexity not justified.
- Constraint: never delete original because derived exists.

## Research C — Fine-Grained Partial Decoding

- Established: chunking, columnar storage, seekable compression.
- Gap: quantifying unnecessary decode in real training workloads; minimizing it.
- Hypothesis: Chunk + field + range granularity reduces bytes decoded per sample request by ≥ an order of magnitude vs whole-dataset decode for random-access workloads, with acceptable index memory.
- Method: implement partial decode; instrument bytes_decoded; compare to baseline.
- Metrics: bytes decoded per request, latency p50/p95/p99, index memory.
- Failure: index overhead dominates; decode savings < 2x.

## Research D — Adaptive Physical Layout Evolution

- Established: materialized views, online reorganization, A/B benchmarking.
- Gap: automatic layout improvement from observed access patterns without downtime.
- Hypothesis: Workload-driven candidate layouts, benchmarked and atomically published, improve target metric (e.g., samples/s) by measurable margin while preserving dataset identity.
- Method: observe access → generate candidate → benchmark offline → verify integrity → atomic publish → keep previous.
- Metrics: samples/s, p99 latency, migration cost, correctness.
- Failure: migration disrupts jobs; gains not statistically significant; rollback frequent.

## Research E — Compression & Scheduling Co-Optimization

- Established: prefetch, worker pools, data loading.
- Gap: jointly optimizing compression plan and distributed worker scheduling.
- Hypothesis: Co-optimizing chunk placement/scheduling with compression plan improves end-to-end training throughput vs optimizing independently.
- Method: model decode+transfer cost; co-optimize; validate on distributed setup.
- Metrics: GPU idle time, throughput, worker imbalance.
- Failure: coupling increases complexity without throughput gain; independent tuning suffices.

## Research F — Learned Lossless Compression

- Established: classical codecs; learned codecs for images/audio; L3/NNCP for text.
- Gap: whether learned lossless helps specific AI dataset families in practice.
- Hypothesis: For at least one dataset family (e.g., structured metadata or token streams), a learned lossless model reduces size vs Zstd while decode throughput remains acceptable, with deterministic decoding.
- Method: prototype; measure ratio + decode throughput + model size + dependency cost.
- Metrics: compression ratio, decode throughput, model load time, determinism, reproducibility.
- Failure (likely): decode too slow; model dependency too heavy; no net benefit. In that case keep classical default and document.

## Training-Aware Reversible Compression (cross-cutting)

What is established: reversible transforms + codecs + chunking.
Integration opportunity: combining them under a training-aware cost model and partial-decode architecture.
Potential original contribution: the closed loop (plan discovery + adaptive layouts + co-optimization + provenance) as an integrated system, if experiments show gains. This is an engineering contribution, not a single algorithmic breakthrough. Must be verified end-to-end.

## Verification standard for any research claim

1. Existing related work cited.
2. Gap stated.
3. Hypothesis falsifiable.
4. Prototype.
5. Baselines.
6. Metrics + raw results published.
7. Failure conditions reported (not hidden).
8. Limitations stated.
9. No claim of novelty without evidence.
