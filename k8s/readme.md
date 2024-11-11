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

### Example 1: With Proxy Configuration

1. **Run `cluster-create.sh` to create the Kind cluster:**
   ```bash
   ./scripts/cluster-create.sh
   ```
2. **Edit CoreDNS Configuration:**

   After the cluster is created, modify the CoreDNS configuration to include DNS rewriting so that traffic to `alpha-sepolia.starknet.io` is routed through the proxy service.

   - Open CoreDNS ConfigMap for editing:

     ```bash
     kubectl -n kube-system edit configmap coredns
     ```

   - Insert the following rewrite rule into the CoreDNS configuration under `Corefile`:

     ```yaml
     rewrite name exact alpha-sepolia.starknet.io proxy.default.svc.cluster.local
     ```

   - Place where to insert rewrite rule:

     ```yaml
     apiVersion: v1
     data:
     Corefile: |
       .:53 {
          errors
          health {
             lameduck 5s
          }
          ready

          rewrite name exact alpha-sepolia.starknet.io server.default.svc.cluster.local

          kubernetes cluster.local in-addr.arpa ip6.arpa {
             pods insecure
             fallthrough in-addr.arpa ip6.arpa
             ttl 30
          }
          prometheus :9153
          forward . /etc/resolv.conf {
             max_concurrent 1000
          }
         
          cache 30
          loop
          reload
          loadbalance
       }
     kind: ConfigMap
     metadata:
     name: coredns
     namespace: kube-system
     ```

   - Save and exit the editor.

   3. **Restart CoreDNS Deployment:**

   Apply the new CoreDNS configuration by restarting the deployment:

   ```bash
   kubectl -n kube-system rollout restart deployment coredns
   ```

   4. **Deploy the Proxy Service:**

   Use the `deploy.sh` script to deploy the proxy service in the Kubernetes cluster:

   ```bash
   ./scripts/deploy.sh
   ```

### Example 1: Without Proxy

In this example, we skip the DNS rewriting step, so requests will not be routed through the proxy.

1. **Run `cluster-run.sh` to create the Kind cluster and deploy pods:**

   ```bash
   ./scripts/cluster-run.sh
   ```
