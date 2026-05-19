FROM rust:1.85-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p axiom -p axctl

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/axiom /usr/local/bin/axiom
COPY --from=builder /app/target/release/axctl /usr/local/bin/axctl
EXPOSE 8080 7946
ENTRYPOINT ["axiom", "run"]
