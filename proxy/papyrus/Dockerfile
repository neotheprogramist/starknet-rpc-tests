FROM ghcr.io/starkware-libs/papyrus:dev

USER root

RUN apk update && apk add --no-cache ca-certificates

COPY ca.crt /usr/local/share/ca-certificates/

RUN update-ca-certificates

