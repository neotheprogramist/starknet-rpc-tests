//! Structures used for deserializing replies from Starkware's sequencer REST
//! API.
use super::header::{self};
use super::state_update as state_update_main;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet_types_core::felt::Felt;

type BlockHash = Felt;
type ContractAddress = Felt;
type GasPrice = u128;
type StateCommitment = Felt;

#[derive(Copy, Clone, Debug, Default, Deserialize, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum L1DataAvailabilityMode {
    #[default]
    Calldata,
    Blob,
}

impl From<L1DataAvailabilityMode> for header::L1DataAvailabilityMode {
    fn from(value: L1DataAvailabilityMode) -> Self {
        match value {
            L1DataAvailabilityMode::Calldata => Self::Calldata,
            L1DataAvailabilityMode::Blob => Self::Blob,
        }
    }
}

impl From<header::L1DataAvailabilityMode> for L1DataAvailabilityMode {
    fn from(value: header::L1DataAvailabilityMode) -> Self {
        match value {
            header::L1DataAvailabilityMode::Calldata => Self::Calldata,
            header::L1DataAvailabilityMode::Blob => Self::Blob,
        }
    }
}

// #[serde_as]
#[derive(Copy, Clone, Debug, Default, Deserialize, PartialEq, Eq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct GasPrices {
    // #[serde_as(as = "GasPriceAsHexStr")]
    pub price_in_wei: GasPrice,
    // #[serde_as(as = "GasPriceAsHexStr")]
    pub price_in_fri: GasPrice,
}

/// Block and transaction status values.
#[derive(Copy, Clone, Default, Debug, Deserialize, PartialEq, Eq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub enum Status {
    #[serde(rename = "NOT_RECEIVED")]
    NotReceived,
    #[serde(rename = "RECEIVED")]
    Received,
    #[serde(rename = "PENDING")]
    Pending,
    #[serde(rename = "REJECTED")]
    Rejected,
    #[serde(rename = "ACCEPTED_ON_L1")]
    AcceptedOnL1,
    #[serde(rename = "ACCEPTED_ON_L2")]
    #[default]
    AcceptedOnL2,
    #[serde(rename = "REVERTED")]
    Reverted,
    #[serde(rename = "ABORTED")]
    Aborted,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::NotReceived => write!(f, "NOT_RECEIVED"),
            Status::Received => write!(f, "RECEIVED"),
            Status::Pending => write!(f, "PENDING"),
            Status::Rejected => write!(f, "REJECTED"),
            Status::AcceptedOnL1 => write!(f, "ACCEPTED_ON_L1"),
            Status::AcceptedOnL2 => write!(f, "ACCEPTED_ON_L2"),
            Status::Reverted => write!(f, "REVERTED"),
            Status::Aborted => write!(f, "ABORTED"),
        }
    }
}

/// Types used when deserializing L2 call related data.
pub mod call {
    use std::collections::HashMap;

    use serde::Deserialize;
    use serde_with::serde_as;

    /// Describes problems encountered during some of call failures .
    #[serde_as]
    #[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct Problems {
        #[serde_as(as = "HashMap<_, _>")]
        pub calldata: HashMap<u64, Vec<String>>,
    }
}

/// Used to deserialize replies to Starknet transaction requests.
///
/// We only care about the statuses so we ignore other fields.
/// Please note that this does not have to be backwards compatible:
/// since we only ever use it to deserialize replies from the Starknet
/// feeder gateway.
#[serde_as]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub struct TransactionStatus {
    pub status: Status,
    pub finality_status: transaction_status::FinalityStatus,
    #[serde(default)]
    pub execution_status: transaction_status::ExecutionStatus,
}

/// Types used when deserializing get_transaction replies.
pub mod transaction_status {
    use serde::Deserialize;

    #[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
    pub enum FinalityStatus {
        #[serde(rename = "NOT_RECEIVED")]
        NotReceived,
        #[serde(rename = "RECEIVED")]
        Received,
        #[serde(rename = "ACCEPTED_ON_L1")]
        AcceptedOnL1,
        #[serde(rename = "ACCEPTED_ON_L2")]
        AcceptedOnL2,
    }

    #[derive(Clone, Copy, Default, Debug, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum ExecutionStatus {
        #[default]
        Succeeded,
        Reverted,
        Rejected,
    }
}

/// Used to deserialize replies to StarkNet state update requests.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StateUpdate {
    /// Gets default value for pending state updates.
    #[serde(default)]
    pub block_hash: BlockHash,
    /// Gets default value for pending state updates.
    #[serde(default)]
    pub new_root: StateCommitment,
    pub old_root: StateCommitment,
    pub state_diff: state_update::StateDiff,
}

impl StateUpdate {
    pub fn new(block_hash: Felt, state_diff: state_update::StateDiff) -> Self {
        // TODO new and old root are not computed, they are not part of the MVP
        Self {
            block_hash,
            new_root: Felt::default(),
            old_root: Felt::default(),
            state_diff,
        }
    }
}

impl From<StateUpdate> for state_update_main::StateUpdate {
    fn from(mut gateway: StateUpdate) -> Self {
        let mut state_update = state_update_main::StateUpdate::default()
            .with_block_hash(gateway.block_hash)
            .with_parent_state_commitment(gateway.old_root)
            .with_state_commitment(gateway.new_root);

        // Extract the known system contract updates from the normal contract updates.
        // This must occur before we map the contract updates, since we want to first
        // remove the system contract updates.
        //
        // Currently this is only the contract at address 0x1.
        //
        // As of starknet v0.12.0 these are embedded in this way, but in the future will
        // be a separate property in the state diff.
        if let Some((address, storage_updates)) = gateway
            .state_diff
            .storage_diffs
            .remove_entry(&ContractAddress::ONE)
        {
            for state_update::StorageDiff { key, value } in storage_updates {
                state_update = state_update.with_system_storage_update(address, key, value);
            }
        }

        // Aggregate contract deployments, storage, nonce and class replacements into
        // contract updates.
        for (address, storage_updates) in gateway.state_diff.storage_diffs {
            for state_update::StorageDiff { key, value } in storage_updates {
                state_update = state_update.with_storage_update(address, key, value);
            }
        }

        for state_update::DeployedContract {
            address,
            class_hash,
        } in gateway.state_diff.deployed_contracts
        {
            state_update = state_update.with_deployed_contract(address, class_hash);
        }

        for (address, nonce) in gateway.state_diff.nonces {
            state_update = state_update.with_contract_nonce(address, nonce);
        }

        for state_update::ReplacedClass {
            address,
            class_hash,
        } in gateway.state_diff.replaced_classes
        {
            state_update = state_update.with_replaced_class(address, class_hash);
        }

        for state_update::DeclaredSierraClass {
            class_hash,
            compiled_class_hash,
        } in gateway.state_diff.declared_classes
        {
            state_update = state_update.with_declared_sierra_class(class_hash, compiled_class_hash);
        }

        state_update.declared_cairo_classes = gateway.state_diff.old_declared_contracts;

        state_update
    }
}

/// Types used when deserializing state update related data.
pub mod state_update {
    use starknet_types_core::felt::Felt;
    use std::collections::{HashMap, HashSet};
    pub type BlockHash = Felt;
    pub type CasmHash = Felt;
    pub type ClassHash = Felt;
    pub type ContractAddress = Felt;
    pub type ContractNonce = Felt;
    pub type SierraHash = Felt;
    pub type StateCommitment = Felt;
    pub type StateDiffCommitment = Felt;
    pub type StorageAddress = Felt;
    pub type StorageValue = Felt;
    use serde::{Deserialize, Serialize};
    use serde_with::serde_as;

    /// L2 state diff.
    #[serde_as]
    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
    #[serde(deny_unknown_fields)]
    pub struct StateDiff {
        #[serde_as(as = "HashMap<_, Vec<_>>")]
        pub storage_diffs: HashMap<ContractAddress, Vec<StorageDiff>>, //
        pub deployed_contracts: Vec<DeployedContract>,
        pub old_declared_contracts: HashSet<ClassHash>, //
        pub declared_classes: Vec<DeclaredSierraClass>,
        pub nonces: HashMap<ContractAddress, ContractNonce>, //
        pub replaced_classes: Vec<ReplacedClass>,
    }

    /// L2 storage diff.
    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
    #[serde(deny_unknown_fields)]
    pub struct StorageDiff {
        pub key: StorageAddress,
        pub value: StorageValue,
    }

    /// L2 contract data within state diff.
    #[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
    #[serde(deny_unknown_fields)]
    pub struct DeployedContract {
        pub address: ContractAddress,
        pub class_hash: ClassHash,
    }

    /// Describes a newly declared class. Maps Sierra class hash to a Casm hash.
    #[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
    #[serde(deny_unknown_fields)]
    pub struct DeclaredSierraClass {
        pub class_hash: SierraHash,
        pub compiled_class_hash: CasmHash,
    }

    /// Describes a newly replaced class. Maps contract address to a new class.
    #[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
    #[serde(deny_unknown_fields)]
    pub struct ReplacedClass {
        pub address: ContractAddress,
        pub class_hash: ClassHash,
    }
}
