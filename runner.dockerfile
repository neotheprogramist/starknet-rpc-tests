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

# Budowanie aplikacji
WORKDIR /usr/src/starknet-rpc-tests/openrpc-testgen-runner
ENV CARGO_TARGET_DIR=/usr/src/starknet-rpc-tests/openrpc-testgen-runner/target

# Budowanie aplikacji z Cargo
RUN cargo build --release --bin openrpc-testgen-runner --features katana

# Final Stage (produkcja)
FROM ubuntu:22.04

# Instalowanie tylko niezbędnych zależności dla uruchomienia aplikacji
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Kopiowanie skompilowanego pliku binarnego
COPY --from=builder /usr/src/starknet-rpc-tests/openrpc-testgen-runner/target/release/openrpc-testgen-runner /usr/local/bin/openrpc-testgen-runner

# Określenie katalogu roboczego
WORKDIR /app

# Domyślny punkt wejścia
ENTRYPOINT ["/usr/local/bin/openrpc-testgen-runner"]

# Opcjonalnie, możesz dodać argumenty do aplikacji (np. --help)
# CMD ["--help"]
