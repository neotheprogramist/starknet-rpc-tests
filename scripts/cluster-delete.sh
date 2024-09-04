#!/usr/bin/env bash

set -euo pipefail

# Delete the cluster
kind delete cluster || true