use fake::Dummy;
use serde::Deserialize;
use starknet_devnet_types::felt::Felt as DevnetFelt;
use starknet_devnet_types::{
    rpc::transaction_receipt::{FeeInUnits, TransactionReceipt},
    traits::ToHexString,
};
use starknet_types_core::felt::Felt;

#[derive(Clone, Deserialize, Default, Debug, PartialEq, Eq)]

pub struct ThinReceipt {
    pub transaction_hash: Felt,
    pub actual_fee: u128,
    pub l2_to_l1_messages: Vec<L2ToL1Message>,
    #[serde(flatten)]
    pub revert_reason: Option<String>,
    pub l1_gas: u128,
    pub l1_data_gas: u128,
}

impl From<Receipt> for ThinReceipt {
    fn from(receipt: Receipt) -> Self {
        ThinReceipt {
            transaction_hash: receipt.transaction_hash,
            actual_fee: u128::from_str_radix(
                receipt.actual_fee.to_hex_string().trim_start_matches("0x"),
                16,
            )
            .unwrap_or(0),
            l2_to_l1_messages: receipt.clone().l2_to_l1_messages,
            revert_reason: receipt.clone().revert_reason().map(|s| s.to_string()),
            l1_gas: receipt.execution_resources.total_gas_consumed.l1_gas,
            l1_data_gas: receipt.execution_resources.total_gas_consumed.l1_data_gas,
        }
    }
}

#[derive(Clone, Deserialize, Default, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub actual_fee: Felt,
    pub execution_resources: ExecutionResources,
    pub l2_to_l1_messages: Vec<L2ToL1Message>,
    pub execution_status: ExecutionStatus,
    pub transaction_hash: Felt,
    pub transaction_index: u64,
}

impl Receipt {
    pub fn is_reverted(&self) -> bool {
        matches!(self.execution_status, ExecutionStatus::Reverted { .. })
    }

    pub fn revert_reason(&self) -> Option<&str> {
        match &self.execution_status {
            ExecutionStatus::Succeeded => None,
            ExecutionStatus::Reverted { reason } => Some(reason.as_str()),
        }
    }
}

impl From<ThinReceipt> for Receipt {
    fn from(receipt: ThinReceipt) -> Self {
        Receipt {
            actual_fee: receipt.actual_fee.into(),
            execution_resources: ExecutionResources {
                total_gas_consumed: L1Gas {
                    l1_gas: receipt.l1_gas,
                    l1_data_gas: receipt.l1_data_gas,
                },
                ..Default::default()
            },
            l2_to_l1_messages: receipt.l2_to_l1_messages,
            execution_status: match receipt.revert_reason {
                Some(reason) => ExecutionStatus::Reverted { reason },
                None => ExecutionStatus::Succeeded,
            },
            transaction_hash: receipt.transaction_hash,
            transaction_index: 0,
        }
    }
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
pub struct L2ToL1Message {
    pub from_address: Felt,
    pub payload: Vec<Felt>,
    // This is purposefully not EthereumAddress even though this
    // represents an Ethereum address normally. Starknet allows this value
    // to be Felt sized; so technically callers can send a message to a garbage
    // address.
    pub to_address: Felt,
}

#[derive(Clone, Deserialize, Debug, Default, PartialEq, Eq)]
pub struct ExecutionResources {
    pub builtins: BuiltinCounters,
    pub n_steps: u64,
    pub n_memory_holes: u64,
    pub data_availability: L1Gas,
    pub total_gas_consumed: L1Gas,
}

#[derive(Clone, Deserialize, Debug, Default, PartialEq, Eq, Dummy)]
pub struct L1Gas {
    pub l1_gas: u128,
    pub l1_data_gas: u128,
}

#[derive(Clone, Deserialize, Debug, Default, PartialEq, Eq)]
pub struct BuiltinCounters {
    pub output: u64,
    pub pedersen: u64,
    pub range_check: u64,
    pub ecdsa: u64,
    pub bitwise: u64,
    pub ec_op: u64,
    pub keccak: u64,
    pub poseidon: u64,
    pub segment_arena: u64,
    pub add_mod: u64,
    pub mul_mod: u64,
    pub range_check96: u64,
}

#[derive(Clone, Deserialize, Default, Debug, PartialEq, Eq, Dummy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionStatus {
    // This must be the default as pre v0.12.1 receipts did not contain this value and
    // were always success as reverted did not exist.
    #[default]
    Succeeded,
    Reverted {
        reason: String,
    },
}

pub fn convert_receipts(old_reveipts: Vec<TransactionReceipt>) -> Vec<ThinReceipt> {
    old_reveipts
        .into_iter()
        .map(|receipt| match receipt {
            TransactionReceipt::Common(tx_receipt) => ThinReceipt {
                transaction_hash: Felt::from_hex_unchecked(
                    tx_receipt.transaction_hash.to_prefixed_hex_str().as_str(),
                ),
                actual_fee: match tx_receipt.actual_fee {
                    FeeInUnits::WEI(fee_amount) => {
                        u128::from_str_radix(&fee_amount.amount.to_string(), 16).unwrap_or(0)
                    }
                    FeeInUnits::FRI(fee_amount) => {
                        u128::from_str_radix(&fee_amount.amount.to_string(), 16).unwrap_or(0)
                    }
                },
                l2_to_l1_messages: tx_receipt
                    .messages_sent
                    .into_iter()
                    .map(|msg| L2ToL1Message {
                        from_address: Felt::from_hex_unchecked(
                            msg.from_address.to_prefixed_hex_str().as_str(),
                        ),
                        to_address: Felt::from_hex_unchecked(
                            DevnetFelt::from(msg.to_address)
                                .to_prefixed_hex_str()
                                .as_str(),
                        ),
                        payload: msg
                            .payload
                            .into_iter()
                            .map(|payload_felt| {
                                Felt::from_hex_unchecked(
                                    payload_felt.to_prefixed_hex_str().as_str(),
                                )
                            })
                            .collect(),
                    })
                    .collect(),
                revert_reason: tx_receipt
                    .execution_status
                    .revert_reason()
                    .map(|s| s.to_string()),
                l1_gas: tx_receipt.execution_resources.data_availability.l1_gas,
                l1_data_gas: tx_receipt.execution_resources.data_availability.l1_data_gas,
            },
            TransactionReceipt::Deploy(tx_receipt) => ThinReceipt {
                transaction_hash: Felt::from_hex_unchecked(
                    tx_receipt
                        .common
                        .transaction_hash
                        .to_prefixed_hex_str()
                        .as_str(),
                ),
                actual_fee: match tx_receipt.common.actual_fee {
                    FeeInUnits::WEI(fee_amount) => {
                        u128::from_str_radix(&fee_amount.amount.to_string(), 16).unwrap_or(0)
                    }
                    FeeInUnits::FRI(fee_amount) => {
                        u128::from_str_radix(&fee_amount.amount.to_string(), 16).unwrap_or(0)
                    }
                },
                l2_to_l1_messages: tx_receipt
                    .common
                    .messages_sent
                    .into_iter()
                    .map(|msg| L2ToL1Message {
                        from_address: Felt::from_hex_unchecked(
                            msg.from_address.to_prefixed_hex_str().as_str(),
                        ),
                        to_address: Felt::from_hex_unchecked(
                            DevnetFelt::from(msg.to_address)
                                .to_prefixed_hex_str()
                                .as_str(),
                        ),
                        payload: msg
                            .payload
                            .into_iter()
                            .map(|payload_felt| {
                                Felt::from_hex_unchecked(
                                    payload_felt.to_prefixed_hex_str().as_str(),
                                )
                            })
                            .collect(),
                    })
                    .collect(),
                revert_reason: tx_receipt
                    .common
                    .execution_status
                    .revert_reason()
                    .map(|s| s.to_string()),
                l1_gas: tx_receipt
                    .common
                    .execution_resources
                    .data_availability
                    .l1_gas,
                l1_data_gas: tx_receipt
                    .common
                    .execution_resources
                    .data_availability
                    .l1_data_gas,
            },
            TransactionReceipt::L1Handler(tx_receipt) => ThinReceipt {
                transaction_hash: Felt::from_hex_unchecked(
                    tx_receipt
                        .common
                        .transaction_hash
                        .to_prefixed_hex_str()
                        .as_str(),
                ),
                actual_fee: match tx_receipt.common.actual_fee {
                    FeeInUnits::WEI(fee_amount) => {
                        u128::from_str_radix(&fee_amount.amount.to_string(), 16).unwrap_or(0)
                    }
                    FeeInUnits::FRI(fee_amount) => {
                        u128::from_str_radix(&fee_amount.amount.to_string(), 16).unwrap_or(0)
                    }
                },
                l2_to_l1_messages: tx_receipt
                    .common
                    .messages_sent
                    .into_iter()
                    .map(|msg| L2ToL1Message {
                        from_address: Felt::from_hex_unchecked(
                            msg.from_address.to_prefixed_hex_str().as_str(),
                        ),
                        to_address: Felt::from_hex_unchecked(
                            DevnetFelt::from(msg.to_address)
                                .to_prefixed_hex_str()
                                .as_str(),
                        ),
                        payload: msg
                            .payload
                            .into_iter()
                            .map(|payload_felt| {
                                Felt::from_hex_unchecked(
                                    payload_felt.to_prefixed_hex_str().as_str(),
                                )
                            })
                            .collect(),
                    })
                    .collect(),
                revert_reason: tx_receipt
                    .common
                    .execution_status
                    .revert_reason()
                    .map(|s| s.to_string()),
                l1_gas: tx_receipt
                    .common
                    .execution_resources
                    .data_availability
                    .l1_gas,
                l1_data_gas: tx_receipt
                    .common
                    .execution_resources
                    .data_availability
                    .l1_data_gas,
            },
        })
        .collect()
}
