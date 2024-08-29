### Create `.cargo/conifg.toml`
* Set envs:
URL = "http://127.0.0.1:5050"

```scarb build```

```cargo build```

# StarkNet Transtaction Tool
## Overview

The StarkNet Transaction Tool is a command-line utility designed to ensure the integrity and correctness of transactions on the StarkNet network. This tool performs two critical functions:

1. **Signature Verification**: It verifies the cryptographic signature of the transaction to ensure that it has been signed by the correct private key corresponding to the provided public key.

2. **Transaction Structure Validation**: It checks the structure of the transaction JSON file to ensure it conforms to the expected format and includes all required fields.

[EXAMPLES](./t9n/README.md)
