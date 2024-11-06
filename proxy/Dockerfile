FROM rust:latest AS builder

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release --package proxy-testgen

RUN cargo build --release --package proxy

FROM ubuntu:22.04

# Install OpenSSL and other necessary libraries
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary 
COPY --from=builder /usr/src/app/target/release/proxy /usr/local/bin/proxy

WORKDIR /usr/src/app

# Copy certificates 
COPY --from=builder /usr/src/app/proxy/alpha-sepolia-certs ./proxy/alpha-sepolia-certs

# Set environment variable and expose the app port
ARG APP_PORT=3000
ENV APP_PORT=${APP_PORT}
EXPOSE ${APP_PORT}

# Run the binary
CMD ["proxy"]
