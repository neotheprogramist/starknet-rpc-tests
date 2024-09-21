FROM alpine:3.20

RUN apk update && apk add --no-cache \
    sudo \
    bash \
    unzip

WORKDIR /mnt/data

COPY testnet-sepolia.sqlite /mnt/data/original/testnet-sepolia.sqlite