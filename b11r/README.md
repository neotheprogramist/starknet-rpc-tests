# b11r Tool

## Overview

The `b11r` tool is designed to create and validate Starknet blocks from the output of the `t8n` tool. It processes transaction receipts, events, and state changes according to the guidelines specified in the Starknet documentation, ensuring blocks are consistent with the Starknet protocol.




## Usage
```bash
cargo run -p b11r -- \
    --block-header-path b11r/testdata/header.json \
    --transactions-path b11r/testdata/txs.json \
    --receipt-path b11r/testdata/transaction_receipts.json \
    --state-diff-path b11r/testdata/state_diff.json \
```