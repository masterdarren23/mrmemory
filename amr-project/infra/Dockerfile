# AMR API — Multi-stage Rust build
# Build: docker build -t amr-api .
# Run:   docker run -p 8080:8080 --env-file .env amr-api

# ── Build stage ──────────────────────────────────────────────
FROM rust:1.82-slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY api/Cargo.toml api/Cargo.lock* ./
# Cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY api/src ./src
COPY api/migrations ./migrations
RUN cargo build --release

# ── Runtime stage ────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN groupadd -r amr && useradd -r -g amr -s /sbin/nologin amr

COPY --from=builder /app/target/release/amr-server /usr/local/bin/amr-server

USER amr
EXPOSE 8080
ENV LISTEN_ADDR=0.0.0.0:8080

HEALTHCHECK --interval=10s --timeout=3s --retries=3 \
  CMD ["/usr/local/bin/amr-server", "--health"] || exit 1

ENTRYPOINT ["/usr/local/bin/amr-server"]
