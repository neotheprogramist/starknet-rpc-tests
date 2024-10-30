
# HTTPS Server Certificate Setup for Starknet RPC Tests

This guide provides steps to generate certificates for an HTTPS server and configure multiple services (e.g., `juno`, `madara`, `papyrus`, `pathfinder`) with these certificates.

## Prerequisites

Ensure you have OpenSSL installed:
```bash
sudo apt update && sudo apt install openssl
```

## Step 1: Generate Certificates

Go into `proxy/alpha-sepolia-certs` directory.

### Commands

1. **Generate CA Private Key**:
    ```bash
    openssl genrsa -out ca.key 2048
    ```

2. **Create CA Certificate**:
    ```bash
    openssl req -x509 -new -nodes -key ca.key -sha256 -days 3650 -out ca.crt -subj "/CN=alpha-sepolia.starknet.io"
    ```

3. **Generate Server Private Key**:
    ```bash
    openssl genrsa -out server.pem 2048
    ```

4. **Create Server Certificate Signing Request (CSR)**:
    ```bash
    openssl req -new -key server.pem -out server.csr -subj "/CN=alpha-sepolia.starknet.io"
    ```

5. **Sign Server CSR with CA Certificate**:
    ```bash
    openssl x509 -req -in server.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out server.crt
    ```

### Output Files

After running the above commands, you should have the following files in `proxy/alpha-sepolia-certs`:

- `ca.key`: CA private key
- `ca.crt`: CA certificate
- `server.pem`: Server private key
- `server.crt`: Signed server certificate

`ca.crt` file will be used in each service’s Dockerfile.


