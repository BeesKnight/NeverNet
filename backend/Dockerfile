FROM rust:1-slim-bookworm AS builder

ARG APP_NAME=edge-api

WORKDIR /app

ENV CARGO_HOME=/usr/local/cargo \
    CARGO_NET_GIT_FETCH_WITH_CLI=true

COPY --link . .

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    cargo build --release --locked -p "${APP_NAME}" \
    && cp "/app/target/release/${APP_NAME}" "/tmp/${APP_NAME}"

FROM debian:bookworm-slim AS runtime

ARG APP_NAME=edge-api
ENV APP_NAME=${APP_NAME}

RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder "/tmp/${APP_NAME}" "/usr/local/bin/${APP_NAME}"

RUN mkdir -p /app/storage/exports

CMD ["sh", "-c", "/usr/local/bin/${APP_NAME}"]
