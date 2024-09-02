# CHECKER

To run this tool install
[Rust](https://doc.rust-lang.org/cargo/getting-started/installation.html) and
[Scarb](https://docs.swmansion.com/scarb/download.html)

## Build

`scarb build`

`cargo build`

## Usage

First of all install starknet-devnet with specified version

```bash
cargo install starknet-devnet --version 0.0.7
```

Now run starknet-devnet

```bash
starknet-devnet
```

After building the crate, you can use it to check the compatibility between the supported versions of Starknet Devnet.

```bash
cargo run -p checker -- --url http://127.0.0.1:5050/ --sierra-path target/dev/cairo_contracts_HelloStarknet.contract_class.json --casm-path target/dev/cairo_contracts_HelloStarknet.compiled_contract_class.json --version v5
```

For more details, see [checker readme](./checker/README.md)

# t8n Tool

## Overview

The `t8n` tool is designed to process transactions and manage state changes in the Starknet ecosystem. It reads input transactions from a JSON file, processes them, and outputs the resulting state to another JSON file. The tool is flexible and can be configured using environment variables to point to the necessary input and output files.

## Usage

```bash
cargo run -p t8n -- --txns-path t8n/src/starknet/input/txns.json --state-path t8n/src/starknet/output/state.json --acc-path t8n/src/starknet/input/acc.json
```

For more details, see [t8n readme](./t8n/README.md)

# t9n Tool

## Overview

The `t9n` tool is a command-line utility designed to ensure the integrity and correctness of transactions on the StarkNet network. This tool performs two critical functions:

1. **Signature Verification**: It verifies the cryptographic signature of the transaction to ensure that it has been signed by the correct private key corresponding to the provided public key.

2. **Transaction Structure Validation**: It checks the structure of the transaction JSON file to ensure it conforms to the expected format and includes all required fields.

## Example run

`Invoke_V1`

```bash
cargo run -p t9n -- --file-path t9n/examples/invoke/invoke_txn_v1.json --public-key 0x39d9e6ce352ad4530a0ef5d5a18fd3303c3606a7fa6ac5b620020ad681cc33b --chain-id 0x534e5f5345504f4c4941
```

For more examples, see [t9n readme](./t9n/README.md)

## Contact

For any questions or feedback, please open an issue on the GitHub repository.
