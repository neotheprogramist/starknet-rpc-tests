### Create `.cargo/conifg.toml`

- Set envs:
  URL = "http://127.0.0.1:5050"

`scarb build`

`cargo build`

# t8n Tool

## Overview

The `t8n` tool is designed to process transactions and manage state changes in the Starknet ecosystem. It reads input transactions from a JSON file, processes them, and outputs the resulting state to another JSON file. The tool is flexible and can be configured using environment variables to point to the necessary input and output files.\

For more details, see [t8n](./t8n/readme.md)

# t9n Tool

## Overview

The `t9n` tool is a command-line utility designed to ensure the integrity and correctness of transactions on the StarkNet network. This tool performs two critical functions:

1. **Signature Verification**: It verifies the cryptographic signature of the transaction to ensure that it has been signed by the correct private key corresponding to the provided public key.

2. **Transaction Structure Validation**: It checks the structure of the transaction JSON file to ensure it conforms to the expected format and includes all required fields.

For more details, see [t9n](./t9n/README.md)
