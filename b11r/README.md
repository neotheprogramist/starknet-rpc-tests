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
{
	"blocks": {
		"header": {
			"block_hash": "0x78c38d93bf1fda7d058a329dd316dff0091ed820728acf13999e16d80b84d5d",
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
			"timestamp": 1727087394,
			"l1_da_mode": "BLOB",
			"starknet_version": "0.0.0"
		},
		"state_diff": {
			"storage_updates": {
				"0x4718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d": {
					"0x723973208639b7839ce298f7ffea61e3f9533872defd7abdb91023db4658812": "0x5520f2c04000",
					"0x6f623aca324f0203acf31ba7864ddcf5b495bc1e5e9235256dea8dad753327": "0x87a2300ecdd33000825021335a100"
				}
			},
			"address_to_nonce": {
				"0x64b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691": "0x1"
			},
			"address_to_class_hash": {},
			"class_hash_to_compiled_class_hash": {
				"0x71758d40a38f84a81b72c1a35934d88c0686adf52a2e051b6ced05237c7820c": "0x5470ec9868f74334a4b992a4c0e539e8a1a043c95d7c4be4850f904837af2d2"
			},
			"declared_contracts": [
				"0x71758d40a38f84a81b72c1a35934d88c0686adf52a2e051b6ced05237c7820c"
			],
			"cairo_0_declared_contracts": []
		}
	},
	"transactions": [
		{
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
	],
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
					"data": ["0x5520f2c04000", "0x0"]
				}
			],
			"execution_status": "SUCCEEDED",
			"finality_status": "ACCEPTED_ON_L2",
			"block_hash": "0x78c38d93bf1fda7d058a329dd316dff0091ed820728acf13999e16d80b84d5d",
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
}
```

### Example Output in JSON

```json
{
  "header": {
    "hash": "0x745471175699a078cade3269421a1013ce6b551062a07ad016e6bf3dcda33e7",
    "parent_hash": "0xb751935b7c033082e15e41198fdedcbe07a7bc0b097f639fbfd8b239d6b56f",
    "number": 1,
    "timestamp": 1727087394,
    "sequencer_address": "0x1000",
    "state_commitment": "0x0",
    "state_diff_commitment": "0x777eb367a0924c164cef4d23c968da55bdb1d2608e36557b9510b2fb26f1f76",
    "transaction_commitment": "0x7ace63cb95885d5b0181fea2c24ccc99bceefde17b0e5f085f7fb6036e21954",
    "transaction_count": 1,
    "event_commitment": "0x2854f307e5899c93d52326815d2afa72f2cb767c241165b8dd098ae1967b2ad",
    "event_count": 1,
    "state_diff_length": 4,
    "starknet_version": "0.0.0",
    "eth_l1_gas_price": 100000000000,
    "strk_l1_gas_price": 100000000000,
    "eth_l1_data_gas_price": 100000000000,
    "strk_l1_data_gas_price": 100000000000,
    "receipt_commitment": "0x53b2ae218fc8ae3de081e49e52a835129904fc35e5572021c6199c39593e34f",
    "l1_da_mode": "BLOB"
  },
  "transactions": [
    {
      "type": "DECLARE",
      "version": "0x3",
      "account_deployment_data": [],
      "class_hash": "0x71758d40a38f84a81b72c1a35934d88c0686adf52a2e051b6ced05237c7820c",
      "compiled_class_hash": "0x5470ec9868f74334a4b992a4c0e539e8a1a043c95d7c4be4850f904837af2d2",
      "fee_data_availability_mode": "L1",
      "nonce": "0x0",
      "nonce_data_availability_mode": "L1",
      "paymaster_data": [],
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
      "sender_address": "0x64b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691",
      "signature": [
        "0x5b0ba9012fa732abd03ec66b417e1f30521d1dfc8066fcb7c6879c8c280a1cf",
        "0x48ab562c8297d7ead1899c00c26cb5ffcca4b5b6b9f08e336f0662ee5acb302"
      ],
      "tip": "0x0",
      "transaction_hash": "0x13b70ad1014a473cdbd63f6938710696b2975e33949e66f007fb59cb0046a1"
    }
  ]
}
```


This JSON structure represents a simplified example of the data `b11r` would process, including block number, state root, transactions, receipts, and state differences.
