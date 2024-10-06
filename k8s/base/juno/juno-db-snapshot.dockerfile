FROM alpine:3.20

RUN apk update && apk add --no-cache wget tar

WORKDIR /snapshots

RUN wget -O juno_sepolia.tar https://juno-snapshots.nethermind.dev/sepolia/juno_sepolia_v0.11.7_66477.tar && \
    tar -xvf juno_sepolia.tar && \
    rm juno_sepolia.tar

