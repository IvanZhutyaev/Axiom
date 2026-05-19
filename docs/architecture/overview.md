# Axiom architecture overview

Axiom is a unified Rust monorepo: one binary (`axiom`) runs as master, worker, storage, or all-in-one depending on CLI flags.

## Layers

1. **Language** — `aql-syntax`, `aql-compile`, `aql-lsp`
2. **Runtime** — `avm-bytecode`, `avm-runtime`, `avm-gc`, `avm-jit`
3. **Storage** — `axiom-storage-log`, `axiom-storage-lsm`, `axiom-raft`
4. **Network** — `axiom-net`, `axiom-gossip`
5. **Engine** — `axiom-engine`, `axiom-connectors`, `axiom-wasm`, `axiom-ml`
6. **API** — `axiom-api`, `axiom-grpc`, `axctl`
7. **Platform** — `axiom-operator`, Helm charts (phase 3), UI (`ui/`)

## Phase 0 status

- AQL parse/compile → `.axc` → AVM interpreter (`axiom-engine::PipelineRunner`)
- Single-partition log + LSM on local disk with checkpoint markers
- UDP transport (handshake, ACK, fragmentation) + SWIM gossip over sim mesh
- REST `/api/v1/jobs`, `/api/v1/cluster`, `/metrics`; `axctl` cluster/job commands
- 3-node dev cluster via `--seed` and `scripts/dev-cluster.ps1`
