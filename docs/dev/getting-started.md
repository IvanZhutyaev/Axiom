# Getting started

## Build

```bash
cargo build --release
cargo test --workspace
```

## Run all-in-one node

```bash
cargo run -p axiom -- run --role all-in-one --data-dir ./data
```

## Submit a job

```bash
cargo run -p axctl -- job submit --file examples/sensor.aql
```

## UI

```bash
cd ui && npm install && npm run dev
```
