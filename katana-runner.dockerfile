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

ENV URLS="http://127.0.0.1:5050" \
    PAYMASTER_ACCOUNT_ADDRESS="0x127fd5f1fe78a71f8bcd1fec63e3fe2f0486b6ecd5c86a0466c3a21fa5cfcec" \
    PAYMASTER_PRIVATE_KEY="0xc5b2fcab997346f3ea1c00b002ecf6f382c5f9c9659a3894eb783c5320f912" \
    UDC_ADDRESS="0x41a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02bf" \
    ACCOUNT_CLASS_HASH="0x07dc7899aa655b0aae51eadff6d801a58e97dd99cf4666ee59e704249e51adf2"

ENTRYPOINT ["sh", "-c", "exec target/release/openrpc-testgen-runner --urls \"$URLS\" --paymaster-account-address \"$PAYMASTER_ACCOUNT_ADDRESS\" --paymaster-private-key \"$PAYMASTER_PRIVATE_KEY\" --udc-address \"$UDC_ADDRESS\" --account-class-hash \"$ACCOUNT_CLASS_HASH\" \"$@\"", "--"]