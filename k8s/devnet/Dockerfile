FROM alpine

WORKDIR /app

RUN apk add --no-cache curl gcc musl-dev libgcc openssl-dev pkgconfig make build-base perl linux-headers


RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustc --version && cargo --version

RUN cargo install starknet-devnet 

CMD ["starknet-devnet"] 