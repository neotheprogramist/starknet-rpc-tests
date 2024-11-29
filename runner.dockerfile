FROM rust:latest AS builder

WORKDIR /usr/src/starknet-rpc-tests

COPY Cargo.toml Cargo.lock ./

COPY openrpc-testgen-runner openrpc-testgen-runner/

COPY . .

WORKDIR /usr/src/starknet-rpc-tests/openrpc-testgen-runner

ENV CARGO_TARGET_DIR=/usr/src/starknet-rpc-tests/openrpc-testgen-runner/target

RUN cargo build --release --bin openrpc-testgen-runner

FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/starknet-rpc-tests/openrpc-testgen-runner/target/release/openrpc-testgen-runner /usr/local/bin/openrpc-testgen-runner

WORKDIR /app


ENTRYPOINT ["/usr/local/bin/openrpc-testgen-runner"]
