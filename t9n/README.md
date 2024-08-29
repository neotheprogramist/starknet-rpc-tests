# StarkNet Transtaction Tool
## Overview

The StarkNet Transaction Tool is a command-line utility designed to ensure the integrity and correctness of transactions on the StarkNet network. This tool performs two critical functions:

1. **Signature Verification**: It verifies the cryptographic signature of the transaction to ensure that it has been signed by the correct private key corresponding to the provided public key.

2. **Transaction Structure Validation**: It checks the structure of the transaction JSON file to ensure it conforms to the expected format and includes all required fields.



### Example: Validating an `Invoke_V1` Transaction

To validate an `Invoke_V1` transaction, run the following command:

```bash
cargo run --bin t9n -- --file-path t9n/examples/invoke/invoke_txn_v1.json --public-key 0x39d9e6ce352ad4530a0ef5d5a18fd3303c3606a7fa6ac5b620020ad681cc33b --chain-id 0x534e5f5345504f4c4941