FROM alpine:3.20

RUN apk update && apk add --no-cache \
    sudo \
    bash \
    unzip

WORKDIR /mnt/data/pathfinder

COPY testnet-sepolia.sqlite /mnt/data/pathfinder

CMD ["sh", "-c", "mkdir -p /mnt/data/pathfinder-2 && cp /mnt/data/pathfinder/testnet-sepolia.sqlite /mnt/data/pathfinder-2/testnet-sepolia.sqlite && tail -f /dev/null"]
