# ── Build stage ───────────────────────────────────────────────────────
FROM rust:1.88-slim AS builder

WORKDIR /app
COPY . .

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo build --release

# ── Runtime stage ─────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/demodatagen /usr/local/bin/demodatagen

WORKDIR /output

ENTRYPOINT ["demodatagen"]
CMD ["--help"]
