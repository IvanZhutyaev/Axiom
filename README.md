# Axiom

Distributed stream processing platform with AQL DSL, AVM runtime, and exactly-once guarantees.

## Quick start

```bash
cargo build --release
cargo test --workspace
cargo run -p axiom -- run --role all-in-one --data-dir ./data
cargo run -p axctl -- job submit --file examples/sensor.aql
cargo run -p axctl -- cluster status
```

See [docs/dev/getting-started.md](docs/dev/getting-started.md), [docs/admin/install.md](docs/admin/install.md), and [docs/PHASES.md](docs/PHASES.md).

### All phases (scaffold)

| Phase | Focus |
|-------|--------|
| 0 | AQL, AVM, log, LSM, gossip, UDP |
| 1 | Raft, scheduler, exactly-once, connectors, JWT API |
| 2 | GC, JIT stub, ML, Wasm |
| 3 | UI (Monaco + graph), gRPC, K8s CRDs, SDK |
| 4 | CI + chaos workflow + acceptance tests |

```bash
cargo run -p axctl -- cluster up --dev-cargo
cargo run -p axctl -- --server http://127.0.0.1:8080 cluster status
cd ui && npm install && npm run dev
```

## Workspace layout

See [docs/architecture/overview.md](docs/architecture/overview.md).

## License

Dual-licensed under MIT OR Apache-2.0.
