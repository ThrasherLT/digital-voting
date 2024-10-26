# Current version of rustc since the start of development:
FROM rust:1.80.1

RUN rustup default stable

RUN rustup component add clippy
RUN rustup component add rustfmt
RUN rustup component add rust-docs
RUN rustup component add rust-src

# These components are not available through rustup and most github actions,
# so precompiling them here:
RUN cargo install cargo-deny leptosfmt
RUN rustup toolchain install nightly
RUN cargo +nightly install cargo-udeps
RUN cargo install wasm-pack
RUN apt update
RUN apt install -y chromium-driver clang

LABEL version="1.0"
LABEL description="Docker image with Rust nightly and linting tools preinstalled"

ENTRYPOINT ["/bin/bash", "-c"]

HEALTHCHECK --interval=5m --timeout=3s \
  CMD curl -f http://localhost/ || exit 1
