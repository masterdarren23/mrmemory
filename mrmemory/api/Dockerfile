# AMR API — Multi-stage Rust build
FROM rust:latest AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY migrations ./migrations
ENV SQLX_OFFLINE=true
RUN cargo build --release

# ── Runtime stage ────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN groupadd -r amr && useradd -r -g amr -s /sbin/nologin amr

COPY --from=builder /app/target/release/amr-server /usr/local/bin/amr-server

USER amr
EXPOSE 8080
ENV LISTEN_ADDR=0.0.0.0:8080

ENTRYPOINT ["/usr/local/bin/amr-server"]
