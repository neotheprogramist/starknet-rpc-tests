use std::{fmt::Display, str::FromStr};

use super::reply::StateUpdate;
use super::serde_utils::GasPriceAsHexStr;
use crate::pathfinder_types::types::{
    block::{BlockHash, BlockTimestamp, CasmHash, ClassHash, ContractAddress},
    block_builder_input::TransactionHash,
    event::Event,
    header::L1DataAvailabilityMode,
    receipt::Receipt,
    reply::Status,
};
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::v0_7_1::starknet_api_openrpc::{
    DeclareTxn, DeployAccountTxn, InvokeTxn, Txn, TxnWithHash,
};

pub type SequencerAddress = Felt;
pub type Fee = Felt;
pub type TransactionNonce = Felt;
pub type TransactionSignatureElem = Felt;
pub type Tip = Felt;
pub type ResourceAmount = String;
pub type ResourcePricePerUnit = String;
pub type PaymasterDataElem = Felt;
pub type AccountDeploymentDataElem = Felt;
pub type ContractAddressSalt = Felt;
pub type ConstructorParam = Felt;
pub type CallParam = Felt;
pub type EntryPoint = Felt;
pub type BlockNumber = u64;
pub type StateCommitment = Felt;
pub type TransactionCommitment = Felt;
pub type EventCommitment = Felt;
pub type ReceiptCommitment = Felt;
pub type StateDiffCommitment = Felt;
pub type TransactionIndex = u64;
pub type L1ToL2MessageNonce = Felt;
pub type L1ToL2MessagePayloadElem = Felt;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct BlockStateUpdate {
    pub block: Block,
    pub state_update: StateUpdate,
}

#[serde_as]
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct Block {
    pub block_hash: BlockHash,
    pub block_number: BlockNumber,

    pub l1_gas_price: GasPrices,
    pub l1_data_gas_price: GasPrices,

    pub parent_block_hash: BlockHash,
    /// Excluded in blocks prior to Starknet 0.8
    pub sequencer_address: SequencerAddress,
    // Historical blocks (pre v0.11) still use `state_root`.
    #[serde(alias = "state_root")]
    pub state_commitment: StateCommitment,
    pub status: Status,
    pub timestamp: BlockTimestamp,
    #[serde_as(as = "Vec<transaction::Receipt>")]
    pub transaction_receipts: Vec<(Receipt, Vec<Event>)>,
    #[serde_as(as = "Vec<Transaction>")]
    pub transactions: Vec<TxnWithHash<Felt>>,
    /// Version metadata introduced in 0.9.1, older blocks will not have it.
    // #[serde(default)]
    #[serde_as(as = "DisplayFromStr")]
    pub starknet_version: StarknetVersion,

    // Introduced in v0.13.1
    pub transaction_commitment: TransactionCommitment,
    pub event_commitment: EventCommitment,
    pub l1_da_mode: L1DataAvailabilityMode,

    // Introduced in v0.13.2, older blocks don't have these fields.
    pub receipt_commitment: ReceiptCommitment,
    pub state_diff_commitment: StateDiffCommitment,
    pub state_diff_length: u32,
}

pub fn count_events(transaction_receipts: Vec<(Receipt, Vec<Event>)>) -> u32 {
    transaction_receipts
        .iter()
        .map(|(_, events)| events.len() as u32)
        .sum()
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct GasPrice(pub u128);

#[serde_as]
#[derive(Copy, Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GasPrices {
    #[serde_as(as = "GasPriceAsHexStr")]
    pub price_in_wei: GasPrice,
    #[serde_as(as = "GasPriceAsHexStr")]
    pub price_in_fri: GasPrice,
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub enum DaMode {
    #[default]
    L1,
    L2,
}

impl<'de> Deserialize<'de> for DaMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match <u8 as Deserialize>::deserialize(deserializer)? {
            0 => Ok(Self::L1),
            1 => Ok(Self::L2),
            _ => Err(serde::de::Error::custom("invalid data availability mode")),
        }
    }
}

impl Serialize for DaMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            DaMode::L1 => serializer.serialize_u8(0),
            DaMode::L2 => serializer.serialize_u8(1),
        }
    }
}

impl From<DaMode> for starknet_types_rpc::v0_7_1::starknet_api_openrpc::DaMode {
    fn from(value: DaMode) -> Self {
        match value {
            DaMode::L1 => Self::L1,
            DaMode::L2 => Self::L2,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct PendingBlockStateMachine {
    block: PendingBlock,
}

#[serde_as]
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct PendingBlock {
    pub l1_gas_price: GasPrices,
    pub l1_data_gas_price: GasPrices,
    #[serde(default)] // TODO: Needed until the gateway provides the l2 gas price
    pub l2_gas_price: GasPrices,
    #[serde(rename = "parent_block_hash")]
    pub parent_hash: BlockHash,
    pub sequencer_address: SequencerAddress,
    pub status: Status,
    pub timestamp: BlockTimestamp,
    #[serde_as(as = "Vec<transaction::Receipt>")]
    pub transaction_receipts: Vec<(Receipt, Vec<Event>)>,
    pub transactions: Vec<Transaction>,
    #[serde_as(as = "DisplayFromStr")]
    pub starknet_version: StarknetVersion,
    pub l1_da_mode: L1DataAvailabilityMode,
}

/// Represents deserialized L2 transaction data.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum Transaction {
    #[serde(rename = "DECLARE")]
    Declare(DeclareTransaction),
    #[serde(rename = "DEPLOY")]
    // FIXME regenesis: remove Deploy txn type after regenesis
    // We are keeping this type of transaction until regenesis
    // only to support older pre-0.11.0 blocks
    Deploy(DeployTransaction),
    #[serde(rename = "DEPLOY_ACCOUNT")]
    DeployAccount(DeployAccountTransaction),
    #[serde(rename = "INVOKE_FUNCTION")]
    Invoke(InvokeTransaction),
    #[serde(rename = "L1_HANDLER")]
    L1Handler(L1HandlerTransaction),
}

impl Transaction {
    /// Returns hash of the transaction
    pub fn hash(&self) -> TransactionHash {
        match self {
            Transaction::Declare(t) => match t {
                DeclareTransaction::V0(t) => t.transaction_hash,
                DeclareTransaction::V1(t) => t.transaction_hash,
                DeclareTransaction::V2(t) => t.transaction_hash,
                DeclareTransaction::V3(t) => t.transaction_hash,
            },
            Transaction::Deploy(t) => t.transaction_hash,
            Transaction::DeployAccount(t) => match t {
                DeployAccountTransaction::V0V1(t) => t.transaction_hash,
                DeployAccountTransaction::V3(t) => t.transaction_hash,
            },
            Transaction::Invoke(t) => match t {
                InvokeTransaction::V0(t) => t.transaction_hash,
                InvokeTransaction::V1(t) => t.transaction_hash,
                InvokeTransaction::V3(t) => t.transaction_hash,
            },
            Transaction::L1Handler(t) => t.transaction_hash,
        }
    }
}

impl<'de> serde_with::DeserializeAs<'de, TxnWithHash<Felt>> for Transaction {
    fn deserialize_as<D>(deserializer: D) -> Result<TxnWithHash<Felt>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::deserialize(deserializer).map(Into::into)
    }
}

impl From<Transaction> for TxnWithHash<Felt> {
    fn from(value: Transaction) -> Self {
        let transaction_hash = value.hash();
        let txn = match value {
            Transaction::Declare(DeclareTransaction::V0(DeclareTransactionV0V1 {
                class_hash,
                max_fee,
                nonce: _,
                sender_address,
                signature,
                transaction_hash: _,
            })) => Txn::Declare(DeclareTxn::V0(
                starknet_types_rpc::v0_7_1::starknet_api_openrpc::DeclareTxnV0 {
                    class_hash,
                    max_fee,
                    sender_address,
                    signature,
                },
            )),
            Transaction::Declare(DeclareTransaction::V1(DeclareTransactionV0V1 {
                class_hash,
                max_fee,
                nonce,
                sender_address,
                signature,
                transaction_hash: _,
            })) => Txn::Declare(DeclareTxn::V1(
                starknet_types_rpc::v0_7_1::starknet_api_openrpc::DeclareTxnV1 {
                    class_hash,
                    max_fee,
                    nonce,
                    sender_address,
                    signature,
                },
            )),
            Transaction::Declare(DeclareTransaction::V2(DeclareTransactionV2 {
                class_hash,
                max_fee,
                nonce,
                sender_address,
                signature,
                transaction_hash: _,
                compiled_class_hash,
            })) => Txn::Declare(DeclareTxn::V2(
                starknet_types_rpc::v0_7_1::starknet_api_openrpc::DeclareTxnV2 {
                    class_hash,
                    max_fee,
                    nonce,
                    sender_address,
                    signature,
                    compiled_class_hash,
                },
            )),
            Transaction::Declare(DeclareTransaction::V3(DeclareTransactionV3 {
                class_hash,
                nonce,
                nonce_data_availability_mode,
                fee_data_availability_mode,
                resource_bounds,
                tip,
                paymaster_data,
                sender_address,
                signature,
                transaction_hash: _,
                compiled_class_hash,
                account_deployment_data,
            })) => Txn::Declare(DeclareTxn::V3(
                starknet_types_rpc::v0_7_1::starknet_api_openrpc::DeclareTxnV3 {
                    account_deployment_data,
                    class_hash,
                    compiled_class_hash,
                    fee_data_availability_mode: fee_data_availability_mode.into(),
                    nonce,
                    nonce_data_availability_mode: nonce_data_availability_mode.into(),
                    paymaster_data,
                    resource_bounds: resource_bounds.into(),
                    sender_address,
                    signature,
                    tip: tip.to_hex_string(),
                },
            )),
            Transaction::Deploy(DeployTransaction {
                contract_address: _,
                contract_address_salt,
                class_hash,
                constructor_calldata,
                transaction_hash: _,
                version,
            }) => Txn::Deploy(
                starknet_types_rpc::v0_7_1::starknet_api_openrpc::DeployTxn {
                    contract_address_salt,
                    class_hash,
                    constructor_calldata,
                    version: version.0,
                },
            ),
            Transaction::DeployAccount(DeployAccountTransaction::V0V1(
                DeployAccountTransactionV0V1 {
                    contract_address: _,
                    transaction_hash: _,
                    max_fee,
                    version: _,
                    signature,
                    nonce,
                    contract_address_salt,
                    constructor_calldata,
                    class_hash,
                },
            )) => Txn::DeployAccount(DeployAccountTxn::V1(
                starknet_types_rpc::v0_7_1::starknet_api_openrpc::DeployAccountTxnV1 {
                    class_hash,
                    constructor_calldata,
                    contract_address_salt,
                    max_fee,
                    nonce,
                    signature,
                },
            )),
            Transaction::DeployAccount(DeployAccountTransaction::V3(
                DeployAccountTransactionV3 {
                    nonce,
                    nonce_data_availability_mode,
                    fee_data_availability_mode,
                    resource_bounds,
                    tip,
                    paymaster_data,
                    sender_address: _,
                    signature,
                    transaction_hash: _,
                    version: _,
                    contract_address_salt,
                    constructor_calldata,
                    class_hash,
                },
            )) => Txn::DeployAccount(DeployAccountTxn::V3(
                starknet_types_rpc::v0_7_1::starknet_api_openrpc::DeployAccountTxnV3 {
                    class_hash,
                    constructor_calldata,
                    contract_address_salt,
                    fee_data_availability_mode: fee_data_availability_mode.into(),
                    nonce,
                    nonce_data_availability_mode: nonce_data_availability_mode.into(),
                    paymaster_data,
                    resource_bounds: resource_bounds.into(),
                    signature,
                    tip,
                },
            )),
            Transaction::Invoke(InvokeTransaction::V0(InvokeTransactionV0 {
                calldata,
                sender_address,
                entry_point_selector,
                max_fee,
                signature,
                transaction_hash: _,
            })) => Txn::Invoke(InvokeTxn::V0(
                starknet_types_rpc::v0_7_1::starknet_api_openrpc::InvokeTxnV0 {
                    calldata,
                    contract_address: sender_address,
                    entry_point_selector,
                    max_fee,
                    signature,
                },
            )),
            Transaction::Invoke(InvokeTransaction::V1(InvokeTransactionV1 {
                calldata,
                sender_address,
                max_fee,
                signature,
                nonce,
                transaction_hash: _,
            })) => Txn::Invoke(InvokeTxn::V1(
                starknet_types_rpc::v0_7_1::starknet_api_openrpc::InvokeTxnV1 {
                    calldata,
                    sender_address,
                    max_fee,
                    signature,
                    nonce,
                },
            )),
            Transaction::Invoke(InvokeTransaction::V3(InvokeTransactionV3 {
                nonce,
                nonce_data_availability_mode,
                fee_data_availability_mode,
                resource_bounds,
                tip,
                paymaster_data,
                sender_address,
                signature,
                transaction_hash: _,
                calldata,
                account_deployment_data,
            })) => Txn::Invoke(InvokeTxn::V3(
                starknet_types_rpc::v0_7_1::starknet_api_openrpc::InvokeTxnV3 {
                    signature,
                    nonce,
                    nonce_data_availability_mode: nonce_data_availability_mode.into(),
                    fee_data_availability_mode: fee_data_availability_mode.into(),
                    resource_bounds: resource_bounds.into(),
                    tip,
                    paymaster_data,
                    account_deployment_data,
                    calldata,
                    sender_address,
                },
            )),
            Transaction::L1Handler(L1HandlerTransaction {
                contract_address,
                entry_point_selector,
                nonce,
                calldata,
                transaction_hash: _,
                // This should always be zero.
                version,
            }) => Txn::L1Handler(
                starknet_types_rpc::v0_7_1::starknet_api_openrpc::L1HandlerTxn {
                    nonce: u64::from_str_radix(&nonce.to_hex_string(), 16).unwrap_or_default(),
                    version: version.0.to_hex_string(),
                    function_call: starknet_types_rpc::v0_7_1::starknet_api_openrpc::FunctionCall {
                        calldata,
                        contract_address,
                        entry_point_selector,
                    },
                },
            ),
        };
        TxnWithHash {
            transaction: txn,
            transaction_hash,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "version")]
pub enum DeclareTransaction {
    #[serde(rename = "0x0")]
    V0(DeclareTransactionV0V1),
    #[serde(rename = "0x1")]
    V1(DeclareTransactionV0V1),
    #[serde(rename = "0x2")]
    V2(DeclareTransactionV2),
    #[serde(rename = "0x3")]
    V3(DeclareTransactionV3),
}

// #[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DeclareTransactionV0V1 {
    pub class_hash: ClassHash,
    pub max_fee: Fee,
    pub nonce: TransactionNonce,
    pub sender_address: ContractAddress,
    // #[serde_as(as = "Vec<TransactionSignatureElemAsDecimalStr>")]
    // #[serde(default)]
    pub signature: Vec<TransactionSignatureElem>,
    pub transaction_hash: TransactionHash,
}

// #[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DeclareTransactionV2 {
    pub class_hash: ClassHash,
    pub max_fee: Fee,
    pub nonce: TransactionNonce,
    pub sender_address: ContractAddress,
    // #[serde_as(as = "Vec<TransactionSignatureElemAsDecimalStr>")]
    // #[serde(default)]
    pub signature: Vec<TransactionSignatureElem>,
    pub transaction_hash: TransactionHash,
    pub compiled_class_hash: CasmHash,
}

// #[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DeclareTransactionV3 {
    pub class_hash: ClassHash,

    pub nonce: TransactionNonce,
    pub nonce_data_availability_mode: DaMode,
    pub fee_data_availability_mode: DaMode,
    pub resource_bounds: ResourceBounds,
    pub tip: Tip,
    pub paymaster_data: Vec<PaymasterDataElem>,

    pub sender_address: ContractAddress,
    // #[serde_as(as = "Vec<TransactionSignatureElemAsDecimalStr>")]
    // #[serde(default)]
    pub signature: Vec<TransactionSignatureElem>,
    pub transaction_hash: TransactionHash,
    pub compiled_class_hash: CasmHash,

    pub account_deployment_data: Vec<AccountDeploymentDataElem>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ResourceBounds {
    #[serde(rename = "L1_GAS")]
    pub l1_gas: ResourceBound,
    #[serde(rename = "L2_GAS")]
    pub l2_gas: ResourceBound,
}

impl From<ResourceBounds>
    for starknet_types_rpc::v0_7_1::starknet_api_openrpc::ResourceBoundsMapping
{
    fn from(value: ResourceBounds) -> Self {
        Self {
            l1_gas: value.l1_gas.into(),
            l2_gas: value.l2_gas.into(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ResourceBound {
    pub max_amount: ResourceAmount,
    pub max_price_per_unit: ResourcePricePerUnit,
}

impl From<ResourceBound> for starknet_types_rpc::v0_7_1::starknet_api_openrpc::ResourceBounds {
    fn from(value: ResourceBound) -> Self {
        Self {
            max_amount: value.max_amount,
            max_price_per_unit: value.max_price_per_unit,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DeployTransaction {
    pub contract_address: ContractAddress,
    pub contract_address_salt: ContractAddressSalt,
    pub class_hash: ClassHash,
    // #[serde_as(as = "Vec<ConstructorParamAsDecimalStr>")]
    pub constructor_calldata: Vec<ConstructorParam>,
    pub transaction_hash: TransactionHash,
    // #[serde(default = "transaction_version_zero")]
    pub version: TransactionVersion,
}

const fn transaction_version_zero() -> TransactionVersion {
    TransactionVersion::ZERO
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum DeployAccountTransaction {
    V0V1(DeployAccountTransactionV0V1),
    V3(DeployAccountTransactionV3),
}

impl<'de> Deserialize<'de> for DeployAccountTransaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;

        #[serde_as]
        #[derive(Deserialize)]
        struct Version {
            #[serde(default = "transaction_version_zero")]
            pub version: TransactionVersion,
        }

        let v = serde_json::Value::deserialize(deserializer)?;
        let version = Version::deserialize(&v).map_err(de::Error::custom)?;

        match version.version {
            ver if ver == TransactionVersion::ZERO => Ok(Self::V0V1(
                serde_json::from_value(v.clone()).map_err(de::Error::custom)?,
            )),
            ver if ver == TransactionVersion::ONE => Ok(Self::V0V1(
                serde_json::from_value(v.clone()).map_err(de::Error::custom)?,
            )),
            ver if ver == TransactionVersion::THREE => Ok(Self::V3(
                serde_json::from_value(v).map_err(de::Error::custom)?,
            )),
            _ => Err(de::Error::custom("version must be 0, 1, or 3")),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct TransactionVersion(pub Felt);

impl TransactionVersion {
    pub const ZERO: Self = Self(Felt::ZERO);
    pub const ONE: Self = Self(Felt::ONE);
    pub const TWO: Self = Self(Felt::TWO);
    pub const THREE: Self = Self(Felt::THREE);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct StarknetVersion(u8, u8, u8, u8);

impl StarknetVersion {
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        StarknetVersion(a, b, c, d)
    }

    pub fn as_u32(&self) -> u32 {
        u32::from_le_bytes([self.0, self.1, self.2, self.3])
    }

    pub fn from_u32(version: u32) -> Self {
        let [a, b, c, d] = version.to_le_bytes();
        StarknetVersion(a, b, c, d)
    }

    pub const V_0_13_2: Self = Self::new(0, 13, 2, 0);
}

impl FromStr for StarknetVersion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(StarknetVersion::new(0, 0, 0, 0));
        }

        let parts: Vec<_> = s.split('.').collect();
        anyhow::ensure!(
            parts.len() == 3 || parts.len() == 4,
            "Invalid version string, expected 3 or 4 parts but got {}",
            parts.len()
        );

        let a = parts[0].parse()?;
        let b = parts[1].parse()?;
        let c = parts[2].parse()?;
        let d = parts.get(3).map(|x| x.parse()).transpose()?.unwrap_or(0);

        Ok(StarknetVersion(a, b, c, d))
    }
}

impl Display for StarknetVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 == 0 && self.1 == 0 && self.2 == 0 && self.3 == 0 {
            return Ok(());
        }
        if self.3 == 0 {
            write!(f, "{}.{}.{}", self.0, self.1, self.2)
        } else {
            write!(f, "{}.{}.{}.{}", self.0, self.1, self.2, self.3)
        }
    }
}

// #[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DeployAccountTransactionV0V1 {
    pub contract_address: ContractAddress,
    pub transaction_hash: TransactionHash,
    pub max_fee: Fee,
    pub version: TransactionVersion,
    // #[serde_as(as = "Vec<TransactionSignatureElemAsDecimalStr>")]
    pub signature: Vec<TransactionSignatureElem>,
    pub nonce: TransactionNonce,
    pub contract_address_salt: ContractAddressSalt,
    // #[serde_as(as = "Vec<CallParamAsDecimalStr>")]
    pub constructor_calldata: Vec<CallParam>,
    pub class_hash: ClassHash,
}

// #[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DeployAccountTransactionV3 {
    pub nonce: TransactionNonce,
    pub nonce_data_availability_mode: DaMode,
    pub fee_data_availability_mode: DaMode,
    pub resource_bounds: ResourceBounds,
    // #[serde_as(as = "TipAsHexStr")]
    pub tip: Tip,
    pub paymaster_data: Vec<PaymasterDataElem>,

    pub sender_address: ContractAddress,
    // #[serde_as(as = "Vec<TransactionSignatureElemAsDecimalStr>")]
    pub signature: Vec<TransactionSignatureElem>,
    pub transaction_hash: TransactionHash,
    pub version: TransactionVersion,
    pub contract_address_salt: ContractAddressSalt,
    // #[serde_as(as = "Vec<CallParamAsDecimalStr>")]
    pub constructor_calldata: Vec<CallParam>,
    pub class_hash: ClassHash,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "version")]
pub enum InvokeTransaction {
    #[serde(rename = "0x0")]
    V0(InvokeTransactionV0),
    #[serde(rename = "0x1")]
    V1(InvokeTransactionV1),
    #[serde(rename = "0x3")]
    V3(InvokeTransactionV3),
}

// #[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InvokeTransactionV0 {
    // #[serde_as(as = "Vec<CallParamAsDecimalStr>")]
    pub calldata: Vec<CallParam>,
    // `contract_address` is the historic name for this field. `sender_address` was introduced
    // with starknet v0.11. As of April 2024 the historic name is still used in older
    // blocks.
    #[serde(alias = "contract_address")]
    pub sender_address: ContractAddress,
    pub entry_point_selector: EntryPoint,
    pub max_fee: Fee,
    // #[serde_as(as = "Vec<TransactionSignatureElemAsDecimalStr>")]
    pub signature: Vec<TransactionSignatureElem>,
    pub transaction_hash: TransactionHash,
}

/// Represents deserialized L2 invoke transaction v1 data.
// #[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InvokeTransactionV1 {
    // #[serde_as(as = "Vec<CallParamAsDecimalStr>")]
    pub calldata: Vec<CallParam>,
    pub sender_address: ContractAddress,
    pub max_fee: Fee,
    // #[serde_as(as = "Vec<TransactionSignatureElemAsDecimalStr>")]
    pub signature: Vec<TransactionSignatureElem>,
    pub nonce: TransactionNonce,
    pub transaction_hash: TransactionHash,
}

/// Represents deserialized L2 invoke transaction v3 data.
// #[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InvokeTransactionV3 {
    pub nonce: TransactionNonce,
    pub nonce_data_availability_mode: DaMode,
    pub fee_data_availability_mode: DaMode,
    pub resource_bounds: ResourceBounds,
    // #[serde_as(as = "TipAsHexStr")]
    pub tip: Tip,
    pub paymaster_data: Vec<PaymasterDataElem>,

    pub sender_address: ContractAddress,
    // #[serde_as(as = "Vec<TransactionSignatureElemAsDecimalStr>")]
    pub signature: Vec<TransactionSignatureElem>,
    pub transaction_hash: TransactionHash,
    // #[serde_as(as = "Vec<CallParamAsDecimalStr>")]
    pub calldata: Vec<CallParam>,

    pub account_deployment_data: Vec<AccountDeploymentDataElem>,
}

// #[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct L1HandlerTransaction {
    pub contract_address: ContractAddress,
    pub entry_point_selector: EntryPoint,
    // FIXME: remove once starkware fixes their gateway bug which was missing this field.
    #[serde(default)]
    pub nonce: TransactionNonce,
    pub calldata: Vec<CallParam>,
    pub transaction_hash: TransactionHash,
    pub version: TransactionVersion,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct EthereumAddress(pub H160);

pub mod transaction {
    use crate::pathfinder_types::types::{receipt::L1Gas, serde_utils::EthereumAddressAsHexStr};
    use serde::Deserialize;
    use serde_with::serde_as;

    use crate::pathfinder_types::types::{
        block::ContractAddress, block_builder_input::TransactionHash, event::Event,
        receipt::L2ToL1Message,
    };

    use super::{
        EntryPoint, EthereumAddress, Fee, L1ToL2MessageNonce, L1ToL2MessagePayloadElem,
        TransactionIndex,
    };

    #[derive(Copy, Clone, Default, Debug, Deserialize, PartialEq, Eq)]
    #[serde(default)]
    pub struct BuiltinCounters {
        pub output_builtin: u64,
        pub pedersen_builtin: u64,
        pub range_check_builtin: u64,
        pub ecdsa_builtin: u64,
        pub bitwise_builtin: u64,
        pub ec_op_builtin: u64,
        pub keccak_builtin: u64,
        pub poseidon_builtin: u64,
        pub segment_arena_builtin: u64, // TODO REMOVE (?)
        pub add_mod_builtin: u64,
        pub mul_mod_builtin: u64,
        pub range_check96_builtin: u64,
    }

    impl From<BuiltinCounters> for crate::pathfinder_types::types::receipt::BuiltinCounters {
        fn from(value: BuiltinCounters) -> Self {
            // Use deconstruction to ensure these structs remain in-sync.
            let BuiltinCounters {
                output_builtin,
                pedersen_builtin,
                range_check_builtin,
                ecdsa_builtin,
                bitwise_builtin,
                ec_op_builtin,
                keccak_builtin,
                poseidon_builtin,
                segment_arena_builtin,
                add_mod_builtin,
                mul_mod_builtin,
                range_check96_builtin,
            } = value;
            Self {
                output: output_builtin,
                pedersen: pedersen_builtin,
                range_check: range_check_builtin,
                ecdsa: ecdsa_builtin,
                bitwise: bitwise_builtin,
                ec_op: ec_op_builtin,
                keccak: keccak_builtin,
                poseidon: poseidon_builtin,
                segment_arena: segment_arena_builtin,
                add_mod: add_mod_builtin,
                mul_mod: mul_mod_builtin,
                range_check96: range_check96_builtin,
            }
        }
    }

    impl From<crate::pathfinder_types::types::receipt::BuiltinCounters> for BuiltinCounters {
        fn from(value: crate::pathfinder_types::types::receipt::BuiltinCounters) -> Self {
            // Use deconstruction to ensure these structs remain in-sync.
            let crate::pathfinder_types::types::receipt::BuiltinCounters {
                output: output_builtin,
                pedersen: pedersen_builtin,
                range_check: range_check_builtin,
                ecdsa: ecdsa_builtin,
                bitwise: bitwise_builtin,
                ec_op: ec_op_builtin,
                keccak: keccak_builtin,
                poseidon: poseidon_builtin,
                segment_arena: segment_arena_builtin,
                add_mod: add_mod_builtin,
                mul_mod: mul_mod_builtin,
                range_check96: range_check96_builtin,
            } = value;
            Self {
                output_builtin,
                pedersen_builtin,
                range_check_builtin,
                ecdsa_builtin,
                bitwise_builtin,
                ec_op_builtin,
                keccak_builtin,
                poseidon_builtin,
                segment_arena_builtin,
                add_mod_builtin,
                mul_mod_builtin,
                range_check96_builtin,
            }
        }
    }

    // #[derive(Copy, Clone, Debug, Default, Deserialize, PartialEq, Eq)]
    // #[serde(deny_unknown_fields)]
    // pub struct L1Gas {
    //     pub l1_gas: u128,
    //     pub l1_data_gas: u128,
    // }

    #[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct ExecutionResources {
        pub builtin_instance_counter: BuiltinCounters,
        pub n_steps: u64,
        pub n_memory_holes: u64,
        pub data_availability: Option<L1Gas>,
        // Added in Starknet 0.13.2
        pub total_gas_consumed: Option<L1Gas>,
    }

    impl From<ExecutionResources> for crate::pathfinder_types::types::receipt::ExecutionResources {
        fn from(value: ExecutionResources) -> Self {
            Self {
                builtins: value.builtin_instance_counter.into(),
                n_steps: value.n_steps,
                n_memory_holes: value.n_memory_holes,
                data_availability: value.data_availability.unwrap_or_default(),
                total_gas_consumed: value.total_gas_consumed.unwrap_or_default(),
            }
        }
    }

    impl From<crate::pathfinder_types::types::receipt::ExecutionResources> for ExecutionResources {
        fn from(value: crate::pathfinder_types::types::receipt::ExecutionResources) -> Self {
            Self {
                builtin_instance_counter: value.builtins.into(),
                n_steps: value.n_steps,
                n_memory_holes: value.n_memory_holes,
                data_availability: Some(value.data_availability),
                total_gas_consumed: Some(value.total_gas_consumed),
            }
        }
    }

    #[serde_as]
    #[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct L1ToL2Message {
        #[serde_as(as = "EthereumAddressAsHexStr")]
        pub from_address: EthereumAddress,
        // #[serde_as(as = "Vec<L1ToL2MessagePayloadElemAsDecimalStr>")]
        pub payload: Vec<L1ToL2MessagePayloadElem>,
        pub selector: EntryPoint,
        pub to_address: ContractAddress,
        #[serde(default)]
        pub nonce: Option<L1ToL2MessageNonce>,
    }

    #[derive(Clone, Default, Debug, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum ExecutionStatus {
        // This must be the default as pre v0.12.1 receipts did not contain this value and
        // were always success as reverted did not exist.
        #[default]
        Succeeded,
        Reverted,
    }

    #[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct Receipt {
        pub actual_fee: Fee,
        pub events: Vec<Event>,
        pub execution_resources: ExecutionResources,
        pub l1_to_l2_consumed_message: Option<L1ToL2Message>,
        pub l2_to_l1_messages: Vec<L2ToL1Message>,
        pub transaction_hash: TransactionHash,
        pub transaction_index: TransactionIndex,
        // Introduced in v0.12.1
        pub execution_status: ExecutionStatus,
        // Introduced in v0.12.1
        /// Only present if status is [ExecutionStatus::Reverted].
        #[serde(default)]
        pub revert_error: Option<String>,
    }

    impl
        From<(
            crate::pathfinder_types::types::receipt::Receipt,
            Vec<crate::pathfinder_types::types::event::Event>,
        )> for Receipt
    {
        fn from(
            (receipt, events): (
                crate::pathfinder_types::types::receipt::Receipt,
                Vec<crate::pathfinder_types::types::event::Event>,
            ),
        ) -> Self {
            let crate::pathfinder_types::types::receipt::Receipt {
                actual_fee,
                execution_resources,
                l2_to_l1_messages,
                execution_status,
                transaction_hash,
                transaction_index,
            } = receipt;

            let (execution_status, revert_error) = match execution_status {
                crate::pathfinder_types::types::receipt::ExecutionStatus::Succeeded => {
                    (ExecutionStatus::Succeeded, None)
                }
                crate::pathfinder_types::types::receipt::ExecutionStatus::Reverted { reason } => {
                    (ExecutionStatus::Reverted, Some(reason))
                }
            };

            Self {
                actual_fee,
                events,
                execution_resources: execution_resources.into(),
                l1_to_l2_consumed_message: None,
                l2_to_l1_messages: l2_to_l1_messages.into_iter().map(Into::into).collect(),
                transaction_hash,
                transaction_index,
                execution_status,
                revert_error,
            }
        }
    }

    impl<'de>
        serde_with::DeserializeAs<
            'de,
            (
                crate::pathfinder_types::types::receipt::Receipt,
                Vec<crate::pathfinder_types::types::event::Event>,
            ),
        > for Receipt
    {
        fn deserialize_as<D>(
            deserializer: D,
        ) -> Result<
            (
                crate::pathfinder_types::types::receipt::Receipt,
                Vec<crate::pathfinder_types::types::event::Event>,
            ),
            D::Error,
        >
        where
            D: serde::Deserializer<'de>,
        {
            Self::deserialize(deserializer).map(Into::into)
        }
    }

    impl From<Receipt>
        for (
            crate::pathfinder_types::types::receipt::Receipt,
            Vec<crate::pathfinder_types::types::event::Event>,
        )
    {
        fn from(value: Receipt) -> Self {
            use crate::pathfinder_types::types::receipt as common;

            let Receipt {
                actual_fee,
                events,
                execution_resources,
                // This information is redundant as it is already in the transaction itself.
                l1_to_l2_consumed_message: _,
                l2_to_l1_messages,
                transaction_hash,
                transaction_index,
                execution_status,
                revert_error,
            } = value;

            (
                common::Receipt {
                    actual_fee,
                    execution_resources: execution_resources.into(),
                    l2_to_l1_messages: l2_to_l1_messages.into_iter().map(Into::into).collect(),
                    transaction_hash,
                    transaction_index,
                    execution_status: match execution_status {
                        ExecutionStatus::Succeeded => common::ExecutionStatus::Succeeded,
                        ExecutionStatus::Reverted => common::ExecutionStatus::Reverted {
                            reason: revert_error.unwrap_or_default(),
                        },
                    },
                },
                events,
            )
        }
    }
}
