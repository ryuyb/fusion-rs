# syntax=docker/dockerfile:1.4

# Build on the host architecture and cross-compile to the target so we avoid QEMU crashes.
FROM --platform=$BUILDPLATFORM debian:bookworm-slim AS builder

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

ENV DEBIAN_FRONTEND=noninteractive \
    MISE_DATA_DIR="/mise" \
    MISE_CONFIG_DIR="/mise" \
    MISE_CACHE_DIR="/mise/cache" \
    MISE_INSTALL_PATH="/usr/local/bin/mise" \
    PATH="/mise/shims:/root/.cargo/bin:${PATH}"

ARG TARGETARCH
ARG TARGETPLATFORM
ARG BUILDPLATFORM

RUN apt-get update \
    && apt-get -y --no-install-recommends install \
        sudo curl git ca-certificates build-essential pkg-config libssl-dev musl-tools \
    && rm -rf /var/lib/apt/lists/*

RUN curl https://mise.run | sh

WORKDIR /app

COPY mise.toml .
RUN mise trust && mise install \
    && mise use -g "zig@latest" \
    && mise exec -- rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl \
    && mise exec -- cargo install cargo-zigbuild

# Pre-fetch dependencies without hardcoding workspace members; any new crate gets picked up automatically.
RUN --mount=type=bind,source=.,target=/workspace,ro \
    --mount=type=cache,target=/root/.cargo/registry,sharing=locked \
    --mount=type=cache,target=/root/.cargo/git,sharing=locked \
    cd /workspace && mise trust /workspace/mise.toml && mise exec -- cargo fetch --locked

# Build actual binary, handling multi-arch compilation.
COPY . .
RUN --mount=type=cache,target=/root/.cargo/registry,sharing=locked \
    --mount=type=cache,target=/root/.cargo/git,sharing=locked \
    case "$TARGETARCH" in \
        arm64) \
            export CARGO_BUILD_TARGET=aarch64-unknown-linux-musl ;; \
        amd64) \
            export CARGO_BUILD_TARGET=x86_64-unknown-linux-musl ;; \
        *) \
            export CARGO_BUILD_TARGET="" ;; \
    esac && \
    if [ -n "$CARGO_BUILD_TARGET" ]; then \
        # cargo-zigbuild cross-compiles without needing QEMU
        mise exec -- cargo zigbuild --release --bin fusion --target "$CARGO_BUILD_TARGET" -- \
            --config net.git-fetch-with-cli=true && \
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
