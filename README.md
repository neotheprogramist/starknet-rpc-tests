# starknet-rpc-tests

Development Setup Instructions

# Setup different devnet`s versions 
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

## Run devnet
```bash
    starknet-devnet-0.0.5 --port 5050
```
```bash
    starknet-devnet-0.0.6 --port 5051
```
## Checking compatibility

To check compatibility for certain version run

Based on the version of the devnet launched on the url we will se compatibility of its methods. Please execute:

```bash
cargo run -p runner -- --url <url>
```
example url: "http://localhost:5050"

## Runnning tests of Starknet Json Rpc methods

First of all build contract

```bash
    scarb build
```
Launch devnet

```bash
    starknet-devnet --seed 0
```
Run tests 

```bash
    cargo test  
```