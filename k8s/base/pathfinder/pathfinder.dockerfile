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

ENTRYPOINT ["./target/release/pathfinder"]