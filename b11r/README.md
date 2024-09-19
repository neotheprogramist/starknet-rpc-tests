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

### Custom Input

In addition to using the output from `t8n` as the default, you can also specify a custom input file.
The input file must be in JSON format and follow the structure expected by `b11r`.

Here’s an example of how to use a custom input file:

```bash
cargo run -p b11r -- --input-path /path/to/custom_input.json --output-path /path/to/output.json
```

### Example Input in JSON

Below is an example of the input JSON structure that `b11r` expects:

```json
  "blocks": {
    "hash_to_block": {
      "0x4badc2140e8e5b27234eee40804c32cf912805562a6e81fbf69c6078ea7869c": {
        "header": {
          "block_hash": "0x4badc2140e8e5b27234eee40804c32cf912805562a6e81fbf69c6078ea7869c",
          "parent_hash": "0xb751935b7c033082e15e41198fdedcbe07a7bc0b097f639fbfd8b239d6b56f",
          "block_number": 1,
          "l1_gas_price": {
            "price_in_fri": "0x174876e800",
            "price_in_wei": "0x174876e800"
          },
          "l1_data_gas_price": {
            "price_in_fri": "0x174876e800",
            "price_in_wei": "0x174876e800"
          },
          "state_root": "0x0",
          "sequencer": "0x1000",
          "timestamp": 1726734837,
          "l1_da_mode": "BLOB",
          "starknet_version": "0.0.0"
        },
        "transaction_hashes": [
          "0x13b70ad1014a473cdbd63f6938710696b2975e33949e66f007fb59cb0046a1",
          "0x7f3caa9350805d338568ff2d5ea4b2dede06a9eee1f59912fbe85988b28aa6"
        ],
        "status": "ACCEPTED_ON_L2"
      }
    },
    "hash_to_state_diff": {
      "0x4badc2140e8e5b27234eee40804c32cf912805562a6e81fbf69c6078ea7869c": {
        "storage_updates": {
          "0x4718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d": {
            "0x6f623aca324f0203acf31ba7864ddcf5b495bc1e5e9235256dea8dad753327": "0x87a2300ecdd3300080ea270f2b100",
            "0x723973208639b7839ce298f7ffea61e3f9533872defd7abdb91023db4658812": "0x6b8095033000"
          }
        },
        "address_to_nonce": {
          "0x64b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691": "0x2"
        },
        "address_to_class_hash": {
          "0x69d009d13af9b505f57dc5fc359e006cf6754d6420f7adbe8ca07bde0b8e2d4": "0x71758d40a38f84a81b72c1a35934d88c0686adf52a2e051b6ced05237c7820c"
        },
        "class_hash_to_compiled_class_hash": {
          "0x71758d40a38f84a81b72c1a35934d88c0686adf52a2e051b6ced05237c7820c": "0x5470ec9868f74334a4b992a4c0e539e8a1a043c95d7c4be4850f904837af2d2"
        },
        "declared_contracts": [
          "0x71758d40a38f84a81b72c1a35934d88c0686adf52a2e051b6ced05237c7820c"
        ],
        "cairo_0_declared_contracts": []
      }
    }
  },
  "transactions": {
    "0x13b70ad1014a473cdbd63f6938710696b2975e33949e66f007fb59cb0046a1": {
      "inner": {
        "transaction_hash": "0x13b70ad1014a473cdbd63f6938710696b2975e33949e66f007fb59cb0046a1",
        "type": "DECLARE",
        "version": "0x3",
        "signature": [
          "0x5b0ba9012fa732abd03ec66b417e1f30521d1dfc8066fcb7c6879c8c280a1cf",
          "0x48ab562c8297d7ead1899c00c26cb5ffcca4b5b6b9f08e336f0662ee5acb302"
        ],
        "nonce": "0x0",
        "resource_bounds": {
          "l1_gas": {
            "max_amount": "0x771a",
            "max_price_per_unit": "0x22ecb25c00"
          },
          "l2_gas": {
            "max_amount": "0x0",
            "max_price_per_unit": "0x0"
          }
        },
        "tip": "0x0",
        "paymaster_data": [],
        "nonce_data_availability_mode": "L1",
        "fee_data_availability_mode": "L1",
        "sender_address": "0x64b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691",
        "compiled_class_hash": "0x5470ec9868f74334a4b992a4c0e539e8a1a043c95d7c4be4850f904837af2d2",
        "class_hash": "0x71758d40a38f84a81b72c1a35934d88c0686adf52a2e051b6ced05237c7820c",
        "account_deployment_data": []
      }
    }
  },
  "transaction_receipts": [
    {
      "type": "DECLARE",
      "transaction_hash": "0x13b70ad1014a473cdbd63f6938710696b2975e33949e66f007fb59cb0046a1",
      "actual_fee": {
        "unit": "FRI",
        "amount": "0x5520f2c04000"
      },
      "messages_sent": [],
      "events": [
        {
          "from_address": "0x4718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d",
          "keys": [
            "0x99cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9",
            "0x64b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691",
            "0x1000"
          ],
          "data": [
            "0x5520f2c04000",
            "0x0"
          ]
        }
      ],
      "execution_status": "SUCCEEDED",
      "finality_status": "ACCEPTED_ON_L2",
      "block_hash": "0x4badc2140e8e5b27234eee40804c32cf912805562a6e81fbf69c6078ea7869c",
      "block_number": 1,
      "execution_resources": {
        "steps": 3513,
        "memory_holes": 112,
        "range_check_builtin_applications": 75,
        "pedersen_builtin_applications": 16,
        "ec_op_builtin_applications": 3,
        "data_availability": {
          "l1_gas": 0,
          "l1_data_gas": 192
        }
      }
    }
  ]
```

This JSON structure represents a simplified example of the data `b11r` would process, including block number, state root, transactions, receipts, and state differences.
