# syntax=docker/dockerfile:1.4

FROM debian:bookworm-slim AS builder

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

ENV DEBIAN_FRONTEND=noninteractive \
    MISE_DATA_DIR="/mise" \
    MISE_CONFIG_DIR="/mise" \
    MISE_CACHE_DIR="/mise/cache" \
    MISE_INSTALL_PATH="/usr/local/bin/mise" \
    PATH="/mise/shims:${PATH}"

RUN apt-get update \
    && apt-get -y --no-install-recommends install \
        sudo curl git ca-certificates build-essential pkg-config libssl-dev musl-tools jq \
    && rm -rf /var/lib/apt/lists/*

RUN curl https://mise.run | sh

WORKDIR /app

COPY mise.toml .
RUN mise trust && mise install && mise exec -- rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl

# Pre-copy manifests for dependency caching.
COPY Cargo.toml Cargo.lock ./
COPY api/Cargo.toml api/Cargo.toml
COPY entity/Cargo.toml entity/Cargo.toml
COPY migration/Cargo.toml migration/Cargo.toml
COPY live-platform/Cargo.toml live-platform/Cargo.toml

# Create placeholder sources to allow dependency fetching.
RUN mkdir -p src api/src entity/src migration/src live-platform/src \
    && echo "fn main() {}" > src/main.rs \
    && echo "" > api/src/lib.rs \
    && echo "" > entity/src/lib.rs \
    && echo "" > migration/src/lib.rs \
    && echo "" > live-platform/src/lib.rs \
    && mise exec -- cargo fetch

# Build actual binary, handling multi-arch compilation.
COPY . .
ARG TARGETARCH
RUN case "$TARGETARCH" in \
        arm64) \
            export CARGO_BUILD_TARGET=aarch64-unknown-linux-musl ;; \
        amd64) \
            export CARGO_BUILD_TARGET=x86_64-unknown-linux-musl ;; \
        *) \
            export CARGO_BUILD_TARGET="" ;; \
    esac && \
    if [ -n "$CARGO_BUILD_TARGET" ]; then \
        mise exec -- cargo build --release --bin fusion --target "$CARGO_BUILD_TARGET" && \
        cp "target/$CARGO_BUILD_TARGET/release/fusion" /tmp/fusion-bin; \
    else \
        mise exec -- cargo build --release --bin fusion && \
        cp target/release/fusion /tmp/fusion-bin; \
    fi && \
    strip /tmp/fusion-bin

FROM gcr.io/distroless/static-debian12:nonroot AS runtime

ENV APP_HOME=/app \
    FUSION_CONFIG_DIR=/app/config

WORKDIR ${APP_HOME}

COPY --from=builder /tmp/fusion-bin /fusion
COPY config ./config

EXPOSE 8080

ENTRYPOINT ["/fusion", "serve"]
