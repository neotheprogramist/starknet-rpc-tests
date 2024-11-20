# OpenRPC-TestGen

`openrpc-testgen` is a Rust library for creating and managing hierarchical test suites and cases for OpenRPC endpoints. It automates the discovery, registration, and execution of tests through a `build.rs` script and enables users to add custom tests easily.

## Quick Start

1. **Add a Test Suite**:

   - Create a directory under `src/` prefixed with `suite_` (e.g., `suite_example`).
   - Add a `mod.rs` file with:
     - A `TestSuite...` struct implementing `SetupableTrait`.
     - An `include!` directive for the auto-generated code:
       ```rust
       include!(concat!(env!("OUT_DIR"), "/generated_tests_suite_example.rs"));
       ```

2. **Add Test Cases**:

   - Create files in the suite directory prefixed with `test_` (e.g., `test_case_one.rs`).
   - Define a `TestCase` struct and implement `RunnableTrait`.

3. **Run the Suite**:

   - Create a binary to execute the test suite. Example:
     ```rust
     #[tokio::main]
     async fn main() {
         let input = SetupInput { /* populate input */ };
         TestSuiteOpenRpc::run(&input).await.unwrap();
     }
     ```

4. **Build and Run**:
   - Build the project with `cargo build`.
   - Execute the tests using the binary:
     ```bash
     cargo run -- <arguments>
     ```

## Full Documentation

For detailed information, refer to the [full README](./openrpc-testgen/readme.md).

# t8n Tool

## Overview

The `t8n` tool is designed to process transactions and manage state changes in the Starknet ecosystem. It reads input transactions from a JSON file, processes them, and outputs the resulting state to another JSON file. The tool is flexible and can be configured using environment variables to point to the necessary input and output files.

## Usage

### Initial State Mode

Use the initial state mode to start with a fresh state:

```bash
cargo run -p t8n -- --txns-path t8n/src/starknet/input/txns.json --state-path target/t8n/output.json --acc-path t8n/src/starknet/input/acc.json
```

### Forwarded State Mode

You can use the forwarded state mode to initialize the state from a previous t8n run:

```bash
cargo run -p t8n -- --txns-path t8n/src/starknet/input/txns_2.json --state-path target/t8n/output.json --forwarded-state
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

# b11r Tool

## Overview

The `b11r` tool is designed to create and validate Starknet blocks from the output of the `t8n` tool.
It processes transaction receipts, events, and state changes according to the guidelines specified in
the Starknet documentation, ensuring blocks are consistent with the Starknet protocol.

By default, the `b11r` tool can receive output directly from the `t8n` tool. **Before running `b11r`,
you first need to run the `t8n` tool to generate the required input.** You can also specify a custom input
file in JSON format, as described below.

## Usage

### Default Input from `t8n`

By default, the `b11r` tool expects input from the output of the `t8n` tool. Make sure you run `t8n`
first to generate the state and transaction data. You can follow the instructions on how to run the `t8n` tool here:

[t8n README - How to Run](../t8n/README.md)

Once the `t8n` tool has been run, you can execute `b11r` like this:

```bash
cargo run -p b11r
```

For more details, visit [b11r readme](./b11r/README.md)

## Contact

For any questions or feedback, please open an issue on the GitHub repository.
