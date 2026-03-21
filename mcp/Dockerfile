# ── Stage 1: build ────────────────────────────────────────────────────────────
FROM rust:1.87-slim AS builder

WORKDIR /build

# Cache dependency compilation
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main(){}' > src/main.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src

# Build the real binary
COPY src ./src
RUN touch src/main.rs && cargo build --release

# ── Stage 2: minimal runtime image ────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/velum-mcp /usr/local/bin/velum-mcp

ENV VELUM_URL=http://velum:3000
ENV VELUM_API_TOKEN=""
ENV MCP_TRANSPORT=stdio
ENV MCP_PORT=8500
ENV RUST_LOG=velum_mcp=info

EXPOSE 8500

ENTRYPOINT ["/usr/local/bin/velum-mcp"]
