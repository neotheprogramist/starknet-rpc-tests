# t8n Tool

## Overview

The `t8n` tool is designed to process transactions and manage state changes in the Starknet ecosystem. It reads input transactions from a JSON file, processes them, and outputs the resulting state to another JSON file. The tool is flexible and can be configured using environment variables to point to the necessary input and output files.

## Prerequisites

Before running the `t8n` tool, you need to set up the necessary environment variables. These environment variables should be defined in the `.cargo/config.toml` file.

### Required Environment Variables

- `ACC_PATH`: Path to the JSON file containing account details used during transaction processing.
- `TXNS_PATH`: Path to the JSON file containing the list of transactions to be processed.
- `STATE_PATH`: Path to the JSON file where the resulting state will be stored after processing the transactions.

### Example Configuration in `.cargo/config.toml`

```toml
[env]
TXNS_PATH = "t8n/src/starknet/input/txns.json"
STATE_PATH = "t8n/src/starknet/output/state.json"
ACC_PATH = "t8n/src/starknet/input/acc.json"
```
