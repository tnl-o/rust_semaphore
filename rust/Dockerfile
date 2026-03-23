# Dockerfile для Velum (Rust)

FROM rust:1.75-alpine3.19 AS builder

# Установка зависимостей
RUN apk add --no-cache -U \
    musl-dev \
    openssl-dev \
    pkgconfig \
    cmake \
    make \
    g++ \
    git

# Установка cargo-chef для оптимизации кэширования
RUN cargo install cargo-chef --locked

WORKDIR /app

# Копирование Cargo.toml и загрузка зависимостей
COPY Cargo.toml Cargo.lock ./
RUN cargo chef prepare --recipe-path recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Копирование исходного кода и сборка
COPY . .
RUN cargo build --release

# Финальный образ
FROM alpine:3.19

# Установка runtime-зависимостей
RUN apk add --no-cache -U \
    ca-certificates \
    tzdata \
    openssl-libs \
    git \
    openssh-client \
    python3 \
    py3-pip \
    && rm -rf /var/cache/apk/*

# Создание пользователя
RUN addgroup -g 1001 velum && \
    adduser -D -u 1001 -G velum velum

# Установка Ansible
RUN pip3 install --break-system-packages ansible

# Копирование бинарного файла
COPY --from=builder /app/target/release/velum /usr/local/bin/

# Создание директорий
RUN mkdir -p /var/lib/velum && \
    mkdir -p /etc/velum && \
    chown -R velum:velum /var/lib/velum && \
    chown -R velum:velum /etc/velum

WORKDIR /var/lib/velum

USER velum

EXPOSE 3000

ENTRYPOINT ["/usr/local/bin/velum"]
CMD ["server"]
