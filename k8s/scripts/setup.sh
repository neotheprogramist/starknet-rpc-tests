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

kind create cluster
if [ $? -ne 0 ]; then
  echo "Error - Cluster not created."
  exit 1
fi

wait_for_service_account() {
  local retries=30
  local count=0
  while [ $count -lt $retries ]; do
    kubectl get serviceaccount default &> /dev/null
    if [ $? -eq 0 ]; then
      echo "Default service account created."
      return 0
    fi
    echo "Waiting for default service account to be created..."
    sleep 2
    count=$((count + 1))
  done
  return 1
}

wait_for_service_account
if [ $? -ne 0 ]; then
  echo "Error - Default service account not found."
  cleanup
fi

echo "Applying pods from $PODS_FILE..."
kubectl apply -f $PODS_FILE
if [ $? -ne 0 ]; then
  echo "Error - Pods not applied."
  cleanup
fi

echo "Applying services from $SERVICES_FILE..."
kubectl apply -f $SERVICES_FILE
if [ $? -ne 0 ]; then
  echo "Error - Services not applied."
  cleanup
fi

echo "Success!"