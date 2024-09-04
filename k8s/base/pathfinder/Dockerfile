FROM rust:1.80.1-bookworm

WORKDIR /app

RUN apt-get update && apt-get install -y \
    build-essential \
    perl \
    git \
    curl \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    zstd \
    && apt-get clean

RUN git clone https://github.com/eqlabs/pathfinder.git

WORKDIR /app/pathfinder

RUN git fetch && git checkout $(git describe --tags `git rev-list --tags --max-count=1`)

RUN cargo build --release --bin pathfinder

ENV PATHFINDER_MONITOR_ADDRESS=0.0.0.0:9090
ENV PATHFINDER_HTTP_RPC_ADDRESS=0.0.0.0:9545
ENV PATHFINDER_ETHEREUM_API_URL=https://eth-sepolia.g.alchemy.com/v2/bkM2DYyg8UBW7zANvO_F9NcagmiqAHRG
ENV PATHFINDER_NETWORK=sepolia-testnet

ENTRYPOINT ["./target/release/pathfinder"]