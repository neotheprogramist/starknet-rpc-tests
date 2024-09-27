#!/usr/bin/env bash

set -euo pipefail

# Create the cluster using kind
kind create cluster --config kind/kind.yaml

# Ensure the default service account exists
kubectl create serviceaccount default || true
kubectl create clusterrolebinding default-admin --clusterrole=cluster-admin --serviceaccount=default:default || true

