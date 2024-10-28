FROM docker.io/piotr439/pathfinder:0_7_1

USER root

COPY ca.crt /usr/local/share/ca-certificates/ca.crt

RUN update-ca-certificates

USER pathfinder

