#!/usr/bin/env bash

set -euo pipefail

# Delete the existing Kubernetes resources
kubectl delete --wait --ignore-not-found -k k8s/base

# Apply Kubernetes manifests
kubectl apply --wait -k k8s/base