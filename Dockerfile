FROM rust:bullseye AS builder

WORKDIR /app

ARG TRUNK_VERSION="v0.16.0"
ARG TRUNK_PATH="/usr/local/bin/trunk"
RUN set -eux; \
    wget -qO- "https://github.com/thedodd/trunk/releases/download/${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz" | tar -xzf-; \
    mv ./trunk $TRUNK_PATH

COPY . .
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/app/crates/client/target \
    --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    --mount=type=cache,target=/root/.rustup \
    set -eux; \
    rustup target install wasm32-unknown-unknown; \
    cargo build --release; \
    objcopy --compress-debug-sections target/release/spotify-banger-backend ./spotify-banger-backend

FROM debian:bullseye-slim

WORKDIR /app

COPY --from=builder /app/spotify-banger-backend ./spotify-banger-backend
CMD ["./spotify-banger-backend"]