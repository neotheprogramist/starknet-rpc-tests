FROM rust:1.80.1-alpine3.19

WORKDIR /app

RUN apk add --no-cache build-base perl 

RUN cargo install starknet-devnet --version 0.1.2

CMD ["starknet-devnet"] 