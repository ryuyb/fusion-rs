# syntax=docker/dockerfile:1.4

FROM --platform=$BUILDPLATFORM rust:1.91.1-alpine3.22 AS chef

RUN set -x \
    && apk add --no-cache musl-dev perl make pkgconfig openssl-dev openssl-libs-static wget curl ca-certificates

RUN cargo install cargo-chef --locked

WORKDIR /app

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM --platform=$BUILDPLATFORM rust:1.91.1-alpine3.22 AS chef-cook

ARG TARGETARCH

RUN set -x \
    && apk add --no-cache musl-dev perl make pkgconfig openssl-dev openssl-libs-static wget curl ca-certificates

COPY --from=chef /usr/local/cargo/bin/cargo-chef /usr/local/cargo/bin/cargo-chef

WORKDIR /app

COPY --from=chef /app/recipe.json recipe.json

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    set -x \
    && case "$TARGETARCH" in \
        "amd64") \
            RUST_TARGET="x86_64-unknown-linux-musl" \
            && MUSL="x86_64-linux-musl" \
            && SHA512="52abd1a56e670952116e35d1a62e048a9b6160471d988e16fa0e1611923dd108a581d2e00874af5eb04e4968b1ba32e0eb449a1f15c3e4d5240ebe09caf5a9f3" ;; \
        "arm64") \
            RUST_TARGET="aarch64-unknown-linux-musl" \
            && MUSL="aarch64-linux-musl" \
            && SHA512="8695ff86979cdf30fbbcd33061711f5b1ebc3c48a87822b9ca56cde6d3a22abd4dab30fdcd1789ac27c6febbaeb9e5bde59d79d66552fae53d54cc1377a19272" ;; \
        *) \
            echo "Unsupported TARGETARCH=$TARGETARCH" && exit 1 ;; \
    esac \
    && wget "https://github.com/AaronChen0/musl-cc-mirror/releases/download/2021-09-23/${MUSL}-cross.tgz" \
    && echo "${SHA512}  ${MUSL}-cross.tgz" | sha512sum -c - \
    && tar -xzf "${MUSL}-cross.tgz" -C /root/ \
    && CC="/root/${MUSL}-cross/bin/${MUSL}-gcc" \
    && rustup target add "${RUST_TARGET}" \
    && RUSTFLAGS="-C linker=${CC}" CC="${CC}" cargo chef cook --release --target "${RUST_TARGET}" --recipe-path recipe.json

FROM --platform=$BUILDPLATFORM rust:1.91.1-alpine3.22 AS builder

ARG TARGETARCH

RUN set -x \
    && apk add --no-cache musl-dev perl make pkgconfig openssl-dev openssl-libs-static wget curl ca-certificates

WORKDIR /app

# Reuse cargo-chef binary to avoid reinstalling
COPY --from=chef /usr/local/cargo/bin/cargo-chef /usr/local/cargo/bin/cargo-chef
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    set -x \
    && case "$TARGETARCH" in \
        "amd64") \
            RUST_TARGET="x86_64-unknown-linux-musl" \
            && MUSL="x86_64-linux-musl" \
            && SHA512="52abd1a56e670952116e35d1a62e048a9b6160471d988e16fa0e1611923dd108a581d2e00874af5eb04e4968b1ba32e0eb449a1f15c3e4d5240ebe09caf5a9f3" ;; \
        "arm64") \
            RUST_TARGET="aarch64-unknown-linux-musl" \
            && MUSL="aarch64-linux-musl" \
            && SHA512="8695ff86979cdf30fbbcd33061711f5b1ebc3c48a87822b9ca56cde6d3a22abd4dab30fdcd1789ac27c6febbaeb9e5bde59d79d66552fae53d54cc1377a19272" ;; \
        *) \
            echo "Unsupported TARGETARCH=$TARGETARCH" && exit 1 ;; \
    esac \
    && wget "https://github.com/AaronChen0/musl-cc-mirror/releases/download/2021-09-23/${MUSL}-cross.tgz" \
    && echo "${SHA512}  ${MUSL}-cross.tgz" | sha512sum -c - \
    && tar -xzf "${MUSL}-cross.tgz" -C /root/ \
    && CC="/root/${MUSL}-cross/bin/${MUSL}-gcc" \
    && rustup target add "${RUST_TARGET}" \
    && RUSTFLAGS="-C linker=${CC}" CC="${CC}" cargo build --release --target "${RUST_TARGET}" \
    && cp "target/${RUST_TARGET}/release/fusion" /tmp/fusion-bin

FROM gcr.io/distroless/static-debian12:nonroot AS runtime

ENV APP_HOME=/app \
    FUSION_CONFIG_DIR=/app/config

WORKDIR ${APP_HOME}

COPY --from=builder /tmp/fusion-bin /fusion
COPY config ./config

EXPOSE 8080

ENTRYPOINT ["/fusion", "serve"]
