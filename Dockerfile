# syntax=docker/dockerfile:1

# 1. Rust: the API server binary and the WebAssembly module.
FROM rust:1-trixie AS rust
# Must equal the wasm-bindgen version in Cargo.lock.
ARG WASM_BINDGEN_VERSION=0.2.129
RUN rustup target add wasm32-unknown-unknown \
 && name="wasm-bindgen-${WASM_BINDGEN_VERSION}-$(uname -m)-unknown-linux-musl" \
 && curl -fsSL "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/${WASM_BINDGEN_VERSION}/${name}.tar.gz" \
    | tar -xz -C /usr/local/bin --strip-components=1 "${name}/wasm-bindgen"
WORKDIR /app
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates crates
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked -p au2-rust-server \
 && cargo build --release --locked -p au2-rust-wasm --target wasm32-unknown-unknown \
 && cp target/release/au2-rust-server /usr/local/bin/ \
 && wasm-bindgen target/wasm32-unknown-unknown/release/au2_rust_wasm.wasm \
    --target web --out-dir /app/web/wasm --out-name core

# 2. Frontend: Vite build against the generated module.
FROM node:24-trixie-slim AS web
WORKDIR /app/web
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web .
COPY --from=rust /app/web/wasm wasm
RUN npm run build:web

# 3. Runtime: one binary plus static files.
FROM debian:trixie-slim
COPY --from=rust /usr/local/bin/au2-rust-server /usr/local/bin/
COPY --from=web /app/web/dist /srv/web
ENV BIND_ADDR=0.0.0.0:3000 \
    STATIC_DIR=/srv/web
USER nobody
EXPOSE 3000
CMD ["au2-rust-server"]
