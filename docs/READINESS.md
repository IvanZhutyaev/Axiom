# Axiom readiness matrix (TZ traceability)

Status legend: **Done** | **Partial** | **Not started**

Last updated: plan implementation sprint.

## Summary

| Phase (TZ §6) | Overall |
|---------------|---------|
| 0 Foundation | **Done** (core gates) |
| 1 Distributed | **Partial** (e2e sim + API) |
| 2 Perf + ML | **Partial** (GC/JIT stubs + predict) |
| 3 UI + ecosystem | **Partial** (UI shell + Helm + gRPC streams) |
| 4 Stabilization | **Partial** (CI cross/kind/chaos scaffold) |
| §9 Acceptance | **Partial** (subset automated) |

---

## §3.1 AQL

| Requirement | Status | Notes |
|-------------|--------|-------|
| Pipeline `\|>`, source/sink/filter/map | Done | Parser, compile, run |
| window + watermark | Done | Parse + `WindowBuffer` / `WatermarkState` |
| if/else, match, let | Done | AST, parser, codegen |
| Binary `.axc` v2 | Done | `avm-bytecode/binary_axc.rs` |
| Full type system | Partial | Extended checker |
| Schema evolution in language | Partial | `axiom-schema` |

## §3.2 AVM

| Requirement | Status | Notes |
|-------------|--------|-------|
| Opcodes in interpreter | Done | Core + struct/array/call |
| Job isolation + GC/TLAB | Done | `JobContext`, `avm-gc` |
| LLVM JIT | Partial | Feature `llvm`; trace stub |
| PREDICT | Done | `axiom-ml::Predictor` |

## §3.3 Engine

| Requirement | Status | Notes |
|-------------|--------|-------|
| SWIM + UDP gossip | Done | `axiom-gossip/udp` |
| Distributed barriers | Done | `DistributedBarrier` |
| Exactly-once e2e | Partial | `exactly_once_e2e` test |
| Prometheus extended | Done | watermark, raft lag, latency |

## §3.4 Storage

| Requirement | Status | Notes |
|-------------|--------|-------|
| Log + compression | Done | `compress.rs` lz4/snappy |
| LSM Bloom + compaction | Done | `bloom.rs`, `compaction.rs` |
| Multi-partition log | Partial | `partition.rs` |
| Raft replication | Partial | `replicate.rs` quorum |

## §3.5 Network

| Requirement | Status | Notes |
|-------------|--------|-------|
| UDP transport + gossip | Done | `UdpTransport`, `UdpGossip` |
| Fragmentation | Done | `fragment.rs` |

## §3.6–3.11 (selected)

| Area | Status | Notes |
|------|--------|-------|
| ML train/infer | Partial | linear, GBDT stub, MLP |
| Wasm connectors | Done | wasmtime default feature |
| REST + RBAC | Partial | JWT roles, delete job |
| axctl commands | Done | deploy, schema, connector |
| gRPC streams | Done | metrics stream, pipeline upload |
| K8s operator | Partial | kube feature + Helm templates |
| UI | Partial | OAuth stub, Sugiyama, LSP WS client |
| SDK codegen | Partial | `scripts/gen-sdk.sh` |

## §9 Acceptance

| Criterion | Status | Notes |
|-----------|--------|-------|
| TZ pipeline compile/run | Done | `acceptance.rs`, `pipeline_e2e` |
| 3-node cluster / exactly-once | Partial | sim tests |
| 1B soak | Partial | `scripts/soak_1b.sh` |
| 5-node `cluster up` | Partial | axctl 5-node local |
| MAPE / visual 3-step UI | Not started | needs production ML + UI polish |

---

See [PHASES.md](PHASES.md) for phase narrative (no false ✅).
