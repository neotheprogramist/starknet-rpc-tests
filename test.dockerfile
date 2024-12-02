
# Build Stage
FROM rust:latest AS builder

# Instalacja zależności: curl, git, bash, unzip oraz asdf
RUN apt-get update && apt-get install -y \
    curl \
    git \
    bash \
    unzip \
    && rm -rf /var/lib/apt/lists/*

# Instalacja asdf
RUN git clone https://github.com/asdf-vm/asdf.git ~/.asdf --branch v0.14.1

# Dodanie asdf do zmiennych środowiskowych
ENV PATH="/root/.asdf/bin:/root/.asdf/shims:${PATH}"

# Instalacja pluginu scarb
RUN asdf plugin add scarb

# Instalacja scarb
RUN asdf install scarb 2.8.4
RUN asdf global scarb 2.8.4

# Praca w katalogu projektu
WORKDIR /usr/src/starknet-rpc-tests

# Kopiowanie Cargo.toml i Cargo.lock, aby cache'ować zależności
COPY Cargo.toml Cargo.lock ./

# Kopiowanie reszty projektu
COPY . .

# Budowanie aplikacji z użyciem scarb
RUN scarb build && cargo build --release --features katana
