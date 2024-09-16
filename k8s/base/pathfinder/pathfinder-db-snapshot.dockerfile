FROM alpine:3.20

RUN apk update && apk add --no-cache \
    curl \
    sudo \
    zstd \
    bash \
    unzip

RUN curl https://rclone.org/install.sh | bash

WORKDIR /mnt/data/pathfinder

COPY rclone.conf /root/.config/rclone/rclone.conf

CMD ["sh", "-c", "sudo -v && rclone copy -P pathfinder-snapshots:pathfinder-snapshots/sepolia-testnet_0.14.0_121057_pruned.sqlite.zst . && zstd -T0 -d sepolia-testnet_0.14.0_121057_pruned.sqlite.zst -o testnet-sepolia.sqlite"]