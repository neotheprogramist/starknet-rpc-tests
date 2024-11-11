FROM docker.io/uacias/madara:v1.0.0 

USER root

COPY ca.crt /usr/local/share/ca-certificates/ca.crt

RUN update-ca-certificates

