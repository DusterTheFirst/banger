FROM rust:bullseye AS builder

WORKDIR /app

ARG TRUNK_VERSION="v0.16.0"
ARG TRUNK_PATH="/usr/local/bin/trunk"
RUN set -eux; \
    wget -qO- "https://github.com/thedodd/trunk/releases/download/${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz" | tar -xzf-; \
    mv ./trunk $TRUNK_PATH

RUN set -eux; \
    export DEBIAN_FRONTEND=noninteractive; \
    apt update; \
    apt install --yes --no-install-recommends \
      brotli; \
    apt clean autoclean; \
    apt autoremove --yes; \
    rm -rf /var/lib/{apt,dpkg,cache,log}/;

COPY . .

WORKDIR /app/crates/client
RUN --mount=type=cache,target=/app/crates/client/target \
    --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    --mount=type=cache,target=/root/.rustup \
    set -eux; \
    rustup target install wasm32-unknown-unknown; \
    trunk build --release; \
    find ./dist -type f | xargs brotli --best --keep;

WORKDIR /app
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    --mount=type=cache,target=/root/.rustup \
    set -eux; \
    cargo build --release; \
    objcopy --compress-debug-sections target/release/spotify-banger-backend ./spotify-banger-backend

FROM debian:bullseye-slim

WORKDIR /app

COPY --from=builder /app/spotify-banger-backend ./spotify-banger-backend
COPY --from=builder /app/crates/client/dist ./static

ENV STATIC_FILES=/app/static
CMD ["./spotify-banger-backend"]