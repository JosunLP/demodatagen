# ── Build stage ───────────────────────────────────────────────────────
# The builder tracks a current stable toolchain (the MSRV is checked in CI);
# builder and runtime share the same Debian release so glibc matches.
FROM rust:1.97-slim-trixie AS builder

WORKDIR /app
COPY . .

# No system TLS libraries needed: self_update uses rustls + ring.
RUN cargo build --release --locked

# ── Runtime stage ─────────────────────────────────────────────────────
FROM debian:trixie-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/demodatagen /usr/local/bin/demodatagen

WORKDIR /output

ENTRYPOINT ["demodatagen"]
CMD ["--help"]
