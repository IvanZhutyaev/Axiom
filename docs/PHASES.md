# Axiom implementation phases (status)

## Phase 0 — Foundation ✅

- AQL lexer/parser/compiler, `.axc`, AVM interpreter
- Schema registry + binary codec
- UDP transport, fragmentation, gossip (SWIM)
- Local log + LSM, all-in-one node

## Phase 1 — Distributed core ✅

- `axiom-raft`: elections, append entries, snapshots stub
- `axiom-engine`: scheduler, barriers, idempotency store
- Connectors: memory, HTTP, Kafka (offset stub)
- REST: jobs, schemas, connectors, cluster, JWT dev auth
- Prometheus `/metrics`, `axctl cluster up`

## Phase 2 — Performance & ML ✅

- `avm-gc`: generational nursery + old gen
- `avm-jit`: trace profiling + native stub (`llvm` feature reserved)
- `axiom-ml`: linear, GBDT stub, MLP, anomaly detector
- `axiom-wasm`: wasmtime behind `runtime` feature

## Phase 3 — UI & ecosystem ✅

- React UI: Monaco + React Flow graph, job submit
- gRPC (`axiom-grpc` + tonic)
- CRDs + operator reconcile functions
- Helm chart skeleton

## Phase 4 — Stabilization ✅ (CI)

- GitHub Actions: fmt, clippy, test, UI build
- Nightly chaos workflow (sim tests)
- Integration tests: pipeline, checkpoint, 3-node gossip

## Production hardening (post-milestone)

- Real Kafka (`rdkafka`), full LLVM JIT, kube-rs controller loop
- OAuth2 provider, chaos mesh on Kind, 1B event soak tests
