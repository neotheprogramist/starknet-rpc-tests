#!/bin/bash

DEFAULT_PODS_FILE="k8s/pathfinder/pods.yaml"
DEFAULT_SERVICES_FILE="k8s/pathfinder/service.yaml"

PODS_FILE="${1:-$DEFAULT_PODS_FILE}"
SERVICES_FILE="${2:-$DEFAULT_SERVICES_FILE}"

cleanup() {
  echo "Deleting cluster..."
  kind delete cluster
  exit 1
}

if ! kind create cluster; then
  echo "Error - Cluster not created."
  exit 1
fi

wait_for_service_account() {
  local retries=30
  local count=0
  while [ $count -lt $retries ]; do
    if kubectl get serviceaccount default &> /dev/null; then
      echo "Default service account created."
      return 0
    fi
    echo "Waiting for default service account to be created..."
    sleep 2
    count=$((count + 1))
  done
  return 1
}

if ! wait_for_service_account; then
  echo "Error - Default service account not found."
  cleanup
fi

echo "Applying pods from $PODS_FILE..."
if ! kubectl apply -f "$PODS_FILE"; then
  echo "Error - Pods not applied."
  cleanup
fi

echo "Applying services from $SERVICES_FILE..."
if ! kubectl apply -f "$SERVICES_FILE"; then
  echo "Error - Services not applied."
  cleanup
fi

echo "Success!"