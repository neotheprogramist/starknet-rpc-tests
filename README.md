# starknet-rpc-tests

Development Setup Instructions

## Starting the Development Network

To start the development network (Devnet) with a specific seed, run the following command:

```bash
starknet-devnet --seed 0
```

Running Tests
After starting the Devnet, you can run your tests using cargo. Execute the following command:

```bash
cargo test
```

Running a Specific Test
If you want to run a specific test, such as jsonrpc_add_declare_transaction, use the following command:


```bash
cargo test jsonrpc_add_declare_transaction -- --nocapture
```

## Checking compatibility

To check compatibility for certain version run

```bash
cargo run -p runner -- --vers <version> --url <url>
```

example version: v5
example url: "http://127.0.0.1:5050"



# Setup different devnets versions 
### V6
```bash
cargo install starknet-devnet --version 0.0.6
```

Rename starkner-devnet v6

```bash
mv ~/.cargo/bin/starknet-devnet ~/.cargo/bin/starknet-devnet-0.0.6
```
### V5
```bash
cargo install starknet-devnet --version 0.0.5
```

Rename starkner-devnet v5

```bash
mv ~/.cargo/bin/starknet-devnet ~/.cargo/bin/starknet-devnet-0.0.5
```
