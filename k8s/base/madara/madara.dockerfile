# Stage 1: Build the application
FROM rust:1.81 AS builder

# Install build dependencies
RUN apt-get -y update && \
    apt-get install -y clang git && \
    apt-get autoremove -y; \
    apt-get clean; \
    rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /usr/src/

# Clone the Madara repository
RUN git clone https://github.com/madara-alliance/madara.git
WORKDIR /usr/src/madara/

# Checkout the specific commit
RUN git checkout 6e13ec2e0252232beb5e7495ac600e7e52d7e3a1

# Installing scarb
ENV SCARB_VERSION="v2.8.2"
ENV SCARB_REPO="https://github.com/software-mansion/scarb/releases/download"
ENV PLATFORM="x86_64-unknown-linux-gnu"
ENV SCARB_TARGET="/usr/src/scarb.tar.gz"

RUN curl -fLS -o $SCARB_TARGET \
    $SCARB_REPO/$SCARB_VERSION/scarb-$SCARB_VERSION-$PLATFORM.tar.gz && \
    tar -xz -C /usr/src/ --strip-components=1 -f $SCARB_TARGET && \
    mv /usr/src/bin/scarb /bin

# Build the application in release mode
RUN cargo build --release

# Stage 2: Create the final runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get -y update && \
    apt-get install -y openssl ca-certificates && \
    apt-get autoremove -y; \
    apt-get clean; \
    rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /usr/local/bin

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/madara/target/release/madara .

# chain presets to be mounted at startup
VOLUME /usr/local/bin/crates/primitives/chain_config/presets
VOLUME /usr/local/bin/crates/primitives/chain_config/resources

# Set the entrypoint
ENTRYPOINT ["./madara"]
