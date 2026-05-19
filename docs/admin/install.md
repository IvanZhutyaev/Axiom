# Installation

## Prerequisites

- Rust stable (1.79+), `cargo`, `rustfmt`, `clippy`
- Node.js 20+ for UI (optional)
- Docker / Kubernetes for production deploy (phase 3)

## Build from source

```bash
git clone <repo> axiom && cd axiom
cargo build --release
```

Binaries: `target/release/axiom`, `target/release/axctl`.

## Single-node (development)

```bash
mkdir -p data
cargo run -p axiom -- run --role all-in-one --data-dir ./data
```

API: http://127.0.0.1:8080/health  
Metrics: http://127.0.0.1:8080/metrics

## Three-node dev cluster

Terminal 1:

```bash
cargo run -p axiom -- run --api-bind 127.0.0.1:8080 --gossip-bind 127.0.0.1:7946 --data-dir ./data1
```

Terminal 2:

```bash
cargo run -p axiom -- run --api-bind 127.0.0.1:8081 --gossip-bind 127.0.0.1:7947 --data-dir ./data2 --seed 127.0.0.1:7946
```

Terminal 3:

```bash
cargo run -p axiom -- run --api-bind 127.0.0.1:8082 --gossip-bind 127.0.0.1:7948 --data-dir ./data3 --seed 127.0.0.1:7946
```

Check membership:

```bash
cargo run -p axctl -- --server http://127.0.0.1:8080 cluster status
```

## Docker

```bash
docker build -f deploy/docker/Dockerfile -t axiom:0.1.0 .
docker run -p 8080:8080 -p 7946:7946/udp axiom:0.1.0
```
