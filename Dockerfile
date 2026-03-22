# ============================================================================
# Dockerfile для Velum (Rust backend) — multi-stage, цель < 50 MB
# ============================================================================
# Использование:
#   docker build -f Dockerfile -t velum .
#   docker compose -f docker-compose.prod.yml up -d
# ============================================================================

# ── Зависимости (кэшируются отдельно от исходников) ──────────────────────
FROM rust:slim AS deps

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Копируем только манифесты, чтобы слой зависимостей кэшировался
COPY rust/Cargo.toml rust/Cargo.lock ./

# ── Основная сборка ───────────────────────────────────────────────────────
FROM deps AS builder

COPY rust/ ./

# profile.release уже содержит: strip=true, lto=true, opt-level="z", panic=abort
RUN cargo build --release

# ── Финальный образ (~20 MB base + stripped binary) ───────────────────────
# gcr.io/distroless/cc-debian12:nonroot содержит glibc + libssl + ca-certs,
# работает с динамически слинкованными Rust бинарями без shell / apt.
# nonroot variant: UID=65532, GID=65532
FROM gcr.io/distroless/cc-debian12:nonroot

# Бинарь (уже stripped благодаря profile.release)
COPY --from=builder /app/target/release/velum /usr/local/bin/velum

# Vanilla JS фронтенд
COPY --chown=65532:65532 web/public /app/web/public

WORKDIR /app

EXPOSE 3000

# БД: PostgreSQL (обязательно задать SEMAPHORE_DB_URL через docker-compose или env)
ENV SEMAPHORE_DB_DIALECT=postgres
ENV SEMAPHORE_WEB_PATH=/app/web/public
ENV SEMAPHORE_ADMIN=admin
ENV SEMAPHORE_ADMIN_PASSWORD=admin123
ENV SEMAPHORE_ADMIN_NAME=Administrator
ENV SEMAPHORE_ADMIN_EMAIL=admin@velum.local

CMD ["/usr/local/bin/velum", "server", "--host", "0.0.0.0", "--port", "3000"]
