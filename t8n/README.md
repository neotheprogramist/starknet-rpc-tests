# t8n Tool

## Overview

The `t8n` tool is designed to process transactions and manage state changes in the Starknet ecosystem. It reads input transactions from a JSON file, processes them, and outputs the resulting state to another JSON file. The tool is flexible and can be configured using environment variables to point to the necessary input and output files.

## Usage

```bash
cargo run -p t8n -- --txns-path t8n/src/starknet/input/txns.json --state-path target/t8n/output.json --acc-path t8n/src/starknet/input/acc.json
```

## Notify

Environment variables are also working

### Required Environment Variables

- `ACC_PATH`: Path to the JSON file containing account details used during transaction processing.
- `TXNS_PATH`: Path to the JSON file containing the list of transactions to be processed.
- `STATE_PATH`: Path to the JSON file where the resulting state will be stored after processing the transactions

### Create `.cargo/conifg.toml`

```toml
[env]
TXNS_PATH = "t8n/src/starknet/input/txns.json"
STATE_PATH = "t8n/src/starknet/output/state.json"
ACC_PATH = "t8n/src/starknet/input/acc.json"
```

Now simply run:

```bash
cargo run -p t8n
```

## Contact

For any questions or feedback, please open an issue on the GitHub repository.
