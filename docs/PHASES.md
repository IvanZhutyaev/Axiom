# Axiom implementation phases

**Authoritative readiness:** [READINESS.md](READINESS.md) (per-requirement status).

Phases follow [техническое задание.md](../техническое%20задание.md) §6 (~3.5–4 years).

## Phase 0 — Foundation (in progress)

Target: TZ pipeline on all-in-one; log+LSM restart; gossip over UDP.

- AQL/AVM: parser, compiler, interpreter, `.axc` v1+v2
- Schema codec, local log, LSM with compaction
- UDP transport, SWIM gossip (UDP + sim tests)

## Phase 1 — Distributed core (in progress)

Target: 3-node exactly-once; rdkafka; full REST/axctl.

## Phase 2 — Performance and ML (in progress)

Target: GC in runtime; JIT feature; ML predict; wasm connectors.

## Phase 3 — UI and ecosystem (in progress)

Target: Monaco+LSP; gRPC streams; kube operator; Helm; SDK.

## Phase 4 — Stabilization (in progress)

Target: CI matrix, Kind, chaos workflow, §9 harnesses.
