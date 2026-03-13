FROM rust:1-slim-bookworm AS builder

WORKDIR /app

ENV CARGO_HOME=/usr/local/cargo \
    CARGO_NET_GIT_FETCH_WITH_CLI=true

COPY --link Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    cargo build --release --locked

RUN rm -rf src

COPY --link migrations ./migrations
COPY --link src ./src

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    rm -f /app/target/release/backend \
    && find /app/target/release/deps -maxdepth 1 \( -name 'backend-*' -o -name 'libbackend-*' \) -delete 2>/dev/null || true \
    && cargo build --release --locked \
    && cp /app/target/release/backend /tmp/backend

FROM debian:bookworm-slim AS runtime

RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /tmp/backend /usr/local/bin/backend

RUN mkdir -p /app/storage/exports

EXPOSE 8080

CMD ["backend"]
