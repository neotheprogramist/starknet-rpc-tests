# b11r Tool

## Overview

The `b11r` tool is designed to create and validate Starknet blocks from the output of the `t8n` tool. It processes transaction receipts, events, and state changes according to the guidelines specified in the Starknet documentation, ensuring blocks are consistent with the Starknet protocol.




## Usage
```bash
cargo run -p b11r -- --input-path b11r/testdata/state.json --output-path target/b11r/block.json
```