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

2.  **Compile Cairo Contracts**:  
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

First of all, install `starknet-devnet` with the specified version:

```bash
cargo install starknet-devnet --version 0.0.7
```

Now run `starknet-devnet`:

```bash
starknet-devnet --state-archive-capacity full --dump-on exit --dump-path dumpdir --seed 0
```

After building the crate, you can use it to check the compatibility between the supported versions of Starknet Devnet. Here’s how to run the checker:

```bash
cargo run -p checker -- \
    --url http://127.0.0.1:5050/ \
    --l1-network-url <L1_NETWORK_URL> \
    --sierra-path target/dev/contracts_contracts_sample_contract_1_HelloStarknet.contract_class.json \
    --casm-path target/dev/contracts_contracts_sample_contract_1_HelloStarknet.compiled_contract_class.json \
    --sierra-path-2 target/dev/contracts_contracts_sample_contract_2_HelloStarknet.contract_class.json \
    --casm-path-2 target/dev/contracts_contracts_sample_contract_2_HelloStarknet.compiled_contract_class.json \
    --run-devnet-tests \
    --private-key <PRIVATE_KEY> \
    --account-address <ACCOUNT_ADDRESS> \
    --account-class-hash <ACCOUNT_CLASS_HASH> \
    --erc20-strk-contract-address <ERC20_STRK_CONTRACT_ADDRESS> \
    --erc20-eth-contract-address <ERC20_ETH_CONTRACT_ADDRESS> \
    --amount-per-test <AMOUNT_PER_TEST> \
    --version v7
```

Example for runnig checker (shorter):

```bash
cargo run -p checker -- \
    -u http://127.0.0.1:5050/ \
    -l https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY \
    -s target/dev/contracts_contracts_sample_contract_1_HelloStarknet.contract_class.json \
    -c target/dev/contracts_contracts_sample_contract_1_HelloStarknet.compiled_contract_class.json \
    --sierra-path-2 target/dev/contracts_contracts_sample_contract_2_HelloStarknet.contract_class.json \
    --casm-path-2 target/dev/contracts_contracts_sample_contract_2_HelloStarknet.compiled_contract_class.json \
    -d \
    -p 0x71d7bb07b9a64f6f78ac4c816aff4da9 \
    -a 0x64b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691 \
    --account-class-hash 0x61dac032f228abef9c6626f995015233097ae253a7f72d68552db02f2971b8f \
    -r 0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d \
    -e 0x49D36570D4E46F48E99674BD3FCC84644DDD6B96F7C741B1562B82F9E004DC7 \
    -m 0xfffffffffffffff \
    -v v7
```

**Note on L1 Network URL:**
For the `--l1-network-url` parameter, you can use various Ethereum node providers:

- Alchemy: `https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY`
- Infura: `https://sepolia.infura.io/v3/YOUR_PROJECT_ID`
- QuickNode: `https://YOUR_SUBDOMAIN.quiknode.pro/YOUR_API_KEY/`

## Notify

Environment variables are also working.

### Create `.cargo/config.toml`

- **Environment Variables**:
  - `RUN_DEVNET_TESTS` = `"true"`
  - `L1_NETWORK_URL`= `"https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY"`
  - `TXNS_PATH`= `"/home/filipg/starknet-devnet/starknet-devnet-tests/t8n/src/starknet/input/txns.json"`
  - `ACC_PATH`= `"/home/filipg/starknet-devnet/starknet-devnet-tests/t8n/src/starknet/input/acc.json"`
  - `VERSION`= `"v7"`
  - `SIERRA_PATH`= `"target/dev/contracts_contracts_sample_contract_1_HelloStarknet.contract_class.json"`
  - `CASM_PATH`= `"target/dev/contracts_contracts_sample_contract_1_HelloStarknet.compiled_contract_class.json"`
  - `SIERRA_PATH_2`= `"target/dev/contracts_contracts_sample_contract_2_HelloStarknet.contract_class.json"`
  - `CASM_PATH_2`= `"target/dev/contracts_contracts_sample_contract_2_HelloStarknet.compiled_contract_class.json"`
  - `URL`= `"http://127.0.0.1:5050"`
  - `ACCOUNT_CLASS_HASH`= `"0x61dac032f228abef9c6626f995015233097ae253a7f72d68552db02f2971b8f"`
  - `ERC20_STRK_CONTRACT_ADDRESS`= `"0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d"`
  - `ERC20_ETH_CONTRACT_ADDRESS`= `"0x49D36570D4E46F48E99674BD3FCC84644DDD6B96F7C741B1562B82F9E004DC7"`
  - `PRIVATE_KEY`= `"0x71d7bb07b9a64f6f78ac4c816aff4da9"`
  - `ACCOUNT_ADDRESS`= `"0x64b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691"`
  - `AMOUNT_PER_TEST`= `"0xfffffffffffffff"`

```toml
[env]
Add your variables here
```

Now simply run:

```bash
cargo run -p checker
```

## Contact

For any questions or feedback, please open an issue on the GitHub repository.
