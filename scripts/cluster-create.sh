#!/usr/bin/env bash

set -euo pipefail

# Create the cluster using kind
kind create cluster --config kind/kind.yaml

# Ensure the default service account exists
kubectl create serviceaccount default || true
kubectl create clusterrolebinding default-admin --clusterrole=cluster-admin --serviceaccount=default:default || true

# # Tworzenie katalogów wewnątrz kind-control-plane
# docker exec kind-control-plane bash -c "mkdir -p /mnt/data/pathfinder"
# docker exec kind-control-plane bash -c "mkdir -p /mnt/data/pathfinder-2"

# # Ustawienie właściciela katalogów
# docker exec kind-control-plane bash -c "chown -R 1000:1000 /mnt/data/pathfinder"
# docker exec kind-control-plane bash -c "chown -R 1000:1000 /mnt/data/pathfinder-2"
