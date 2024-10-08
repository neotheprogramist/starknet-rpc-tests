#!/usr/bin/env bash

set -euo pipefail

print_in_blue() {
    echo -e "\033[1;34m$1\033[0m"
}

print_in_blue "Deploy the app..."
./scripts/deploy.sh