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
        sudo curl git ca-certificates build-essential pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN curl https://mise.run | sh

WORKDIR /app

COPY mise.toml .
RUN mise trust && mise install

# Pre-copy manifests for dependency caching.
COPY Cargo.toml Cargo.lock ./
COPY api/Cargo.toml api/Cargo.toml
COPY entity/Cargo.toml entity/Cargo.toml
COPY migration/Cargo.toml migration/Cargo.toml

# Create placeholder sources to allow dependency fetching.
RUN mkdir -p src api/src entity/src migration/src \
    && echo "fn main() {}" > src/main.rs \
    && echo "" > api/src/lib.rs \
    && echo "" > entity/src/lib.rs \
    && echo "" > migration/src/lib.rs \
    && mise exec -- cargo fetch

# Build actual binary, handling multi-arch compilation.
COPY . .
ARG TARGETARCH
RUN case "$TARGETARCH" in \
        arm64) \
            export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc && \
            export CARGO_BUILD_TARGET=aarch64-unknown-linux-gnu ;; \
        amd64) \
            export CARGO_BUILD_TARGET=x86_64-unknown-linux-gnu ;; \
        *) \
            export CARGO_BUILD_TARGET="" ;; \
    esac && \
    if [ -n "$CARGO_BUILD_TARGET" ]; then \
        mise exec -- cargo build --release --bin fusion --target "$CARGO_BUILD_TARGET" && \
        cp "target/$CARGO_BUILD_TARGET/release/fusion" /tmp/fusion-bin; \
    else \
        mise exec -- cargo build --release --bin fusion && \
        cp target/release/fusion /tmp/fusion-bin; \
    fi

FROM gcr.io/distroless/cc-debian12 AS runtime

ENV APP_HOME=/app \
    FUSION_CONFIG_DIR=/app/config

WORKDIR ${APP_HOME}

COPY --from=builder /tmp/fusion-bin /usr/local/bin/fusion
COPY config ./config

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/fusion", "serve"]
