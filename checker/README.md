# Checker

## Overview

The `checker` crate is designed to verify compatibility between different versions of the Starknet Devnet. This tool currently supports compatibility checks between the following versions:
- v0.0.5
- v0.0.6
- v0.0.7

## Setup

To prepare the `checker` crate for use, you need to perform the following steps:
1.  **Install 
[Rust](https://doc.rust-lang.org/cargo/getting-started/installation.html)** and **[Scarb](https://docs.swmansion.com/scarb/download.html)**

2. **Compile Cairo Contracts**:  
   Before using the crate, you need to compile the Cairo contracts. This can be done using the `scarb` tool. Run the following command in your terminal:

   
```bash
   scarb build
```   
3. **Build project**:  
   After successfully compiling the Cairo contracts, proceed to build the `checker` crate using Cargo:

   
```bash
   cargo build
```   

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

## Notify
Environment variables are also working 
### Create `.cargo/conifg.toml`
```toml
[env]
URL="http://127.0.0.1:5050/"
SIERRA_PATH = "target/dev/cairo_contracts_HelloStarknet.contract_class.json"
CASM_PATH = "target/dev/cairo_contracts_HelloStarknet.compiled_contract_class.json"
```
Now simply run:
```bash
cargo run -p checker
```

## Contact

For any questions or feedback, please open an issue on the GitHub repository.