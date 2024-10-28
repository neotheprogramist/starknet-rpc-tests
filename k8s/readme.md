# Primary Hive Equivalent Tool

## Overview (todo)

### Key Features (todo)

## Prerequisites

Ensure that you have the following installed on your system:

- Docker
- Kubernetes (kubectl)
- KinD (Kubernetes in Docker)

## Usage

1. **Create .env file in k8s/base/anvil**

   ```sh
   FORK_URL=FORK_URL
   ```

   **Create .env file in k8s/base/madara**

   ```sh
   MADARA_GATEWAY_KEY=MADARA_GATEWAY_KEY
   ```

   2. **Run the setup script**:

   ```sh
   ./k8s/scripts/cluster-run.sh
   ```
