# StarkNet Transtaction Tool

## Overview

The StarkNet Transaction Tool is a command-line utility designed to ensure the integrity and correctness of transactions on the StarkNet network. This tool performs two critical functions:

1. **Signature Verification**: It verifies the cryptographic signature of the transaction to ensure that it has been signed by the correct private key corresponding to the provided public key.

2. **Transaction Structure Validation**: It checks the structure of the transaction JSON file to ensure it conforms to the expected format and includes all required fields.

## Usage

#### Validating an `INVOKE` Transactions

- `Invoke_V1`

```bash
cargo run -p t9n -- --file-path t9n/examples/invoke/invoke_txn_v1.json --public-key 0x39d9e6ce352ad4530a0ef5d5a18fd3303c3606a7fa6ac5b620020ad681cc33b --chain-id 0x534e5f5345504f4c4941
```

- `Invoke_V3`

```bash
cargo run -p t9n -- --file-path t9n/examples/invoke/invoke_txn_v3.json --public-key 0x39d9e6ce352ad4530a0ef5d5a18fd3303c3606a7fa6ac5b620020ad681cc33b --chain-id 0x534e5f5345504f4c4941
```

#### Validating an `DECLARE` Transactions

- `Declare_V2`

```bash
cargo run -p t9n -- --file-path t9n/examples/declare/declare_txn_v2.json --public-key 0x39d9e6ce352ad4530a0ef5d5a18fd3303c3606a7fa6ac5b620020ad681cc33b --chain-id 0x534e5f5345504f4c4941
```

- `Declare_V3`

```bash
cargo run -p t9n -- --file-path t9n/examples/declare/declare_txn_v3.json --public-key 0x39d9e6ce352ad4530a0ef5d5a18fd3303c3606a7fa6ac5b620020ad681cc33b --chain-id 0x534e5f5345504f4c4941
```

#### Validating an `DEPLOY_ACCOUNT` Transactions

- `DeployAccount_V1`

```bash
cargo run -p t9n -- --file-path t9n/examples/deploy_acc/deploy_acc_txn_v1.json --public-key 0x539751391da90f5789033ecf54ba0bdb4cbad7f92068418e22951e9973c05ea --chain-id 0x534e5f5345504f4c4941
```

- `DeployAccount_V3`

```bash
cargo run -p t9n -- --file-path t9n/examples/deploy_acc/deploy_acc_txn_v3.json --public-key 0x6ac091f93bebf5d88f4905415d9878ad2c1892e8b4a72fa3c3a497df76f3bb0 --chain-id 0x534e5f5345504f4c4941
```

## Notify

Environment variables are also working

### Create `.cargo/conifg.toml`

```toml
[env]
FILE_PATH = "t9n/examples/invoke/invoke_txn_v1.json"
PUBLIC_KEY = "0x39d9e6ce352ad4530a0ef5d5a18fd3303c3606a7fa6ac5b620020ad681cc33b"
CHAIN_ID = "0x534e5f5345504f4c4941"
```

Now simply run:

```bash
cargo run -p t9n
```

## Contact

For any questions or feedback, please open an issue on the GitHub repository.
