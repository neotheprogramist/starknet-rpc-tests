FROM rust:1.82.0-slim-bookworm 

RUN apt-get update && apt-get install -y \
    curl \
    libssl-dev \
    git \
    bash \
    make \
    unzip \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN git clone https://github.com/asdf-vm/asdf.git ~/.asdf --branch v0.14.1

ENV PATH="/root/.asdf/bin:/root/.asdf/shims:${PATH}"

RUN asdf plugin add scarb && asdf install scarb 2.8.4 && asdf global scarb 2.8.4

WORKDIR /usr/src/starknet-rpc-tests

COPY Cargo.toml Cargo.lock ./

COPY . .

RUN scarb build && cargo build --release --features katana 

