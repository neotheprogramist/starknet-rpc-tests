# Primary Hive Equivalent Tool

## Overview (todo)

### Key Features (todo)

## Prerequisites

Ensure that you have the following installed on your system:

- Docker
- Kubernetes (kubectl)
- KinD (Kubernetes in Docker)

## Usage

1. **Run the setup script**:

   ```sh
   ./k8s/scripts/setup.sh
   ```

   By default, the script will use the following files:

   - Pods configuration: [`k8s/pathfinder/pods.yaml`](../k8s/pathfinder/pods.yaml)
   - Services configuration: [`k8s/pathfinder/service.yaml`](../k8s/pathfinder/service.yaml)

   You can also specify custom files as arguments:

   ```sh
   ./k8s/scripts/setup.sh <path-to-pods-file> <path-to-services-file>
   ```
