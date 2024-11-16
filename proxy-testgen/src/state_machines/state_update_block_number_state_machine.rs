use crate::StateMachine;
use crate::StateMachineResult;
use anyhow::Result;
use production_nodes_types::pathfinder_types::types::block_builder_input::TransactionHash;
use production_nodes_types::pathfinder_types::types::block_hash::{
    calculate_event_commitment, calculate_receipt_commitment, calculate_transaction_commitment,
    compute_final_hash,
};
use production_nodes_types::pathfinder_types::types::gateway_state_update::{
    BlockStateUpdate, StarknetVersion,
};
use production_nodes_types::pathfinder_types::types::state_update::{
    state_diff_commitment::compute, StateUpdate,
};
use production_nodes_types::pathfinder_types::types::{
    event::Event, receipt::Receipt, reply::StateUpdate as GatewayStateUpdate,
};
use regex::Regex;
use serde::Deserialize;
use serde_json::Result as JsonResult;
use serde_with::{serde_as, DisplayFromStr};
use starknet_types_core::felt::Felt;

#[derive(Clone)]
pub struct Ok;

#[derive(Clone)]
pub struct Invalid;

#[derive(Clone)]
pub struct Skipped;

#[derive(Clone)]
pub struct StateUpdateBlockNumberStateMachine<S> {
    path: String,
    state: S,
}

impl<S> StateUpdateBlockNumberStateMachine<S> {
    pub fn get_state(&self) -> &S {
        &self.state
    }
}

impl StateUpdateBlockNumberStateMachine<Ok> {
    pub fn new() -> Self {
        Self {
            path: "/feeder_gateway/get_state_update?blockNumber".to_string(),
            state: Ok,
        }
    }

    pub fn to_invalid(self) -> StateUpdateBlockNumberStateMachine<Invalid> {
        StateUpdateBlockNumberStateMachine {
            path: self.path,
            state: Invalid,
        }
    }

    pub fn to_skipped(&self) -> StateUpdateBlockNumberStateMachine<Skipped> {
        StateUpdateBlockNumberStateMachine {
            path: self.path.clone(),
            state: Skipped,
        }
    }
}

impl Default for StateUpdateBlockNumberStateMachine<Ok> {
    fn default() -> Self {
        Self::new()
    }
}

impl StateUpdateBlockNumberStateMachine<Invalid> {
    pub fn to_skipped(self) -> StateUpdateBlockNumberStateMachine<Skipped> {
        StateUpdateBlockNumberStateMachine {
            path: self.path,
            state: Skipped,
        }
    }
}

impl StateUpdateBlockNumberStateMachine<Skipped> {
    pub fn to_okk(self) -> StateUpdateBlockNumberStateMachine<Ok> {
        StateUpdateBlockNumberStateMachine {
            path: self.path,
            state: Ok,
        }
    }
}

pub enum StateUpdateBlockNumberStateMachineWrapper {
    Ok(StateUpdateBlockNumberStateMachine<Ok>),
    Invalid(StateUpdateBlockNumberStateMachine<Invalid>),
    Skipped(StateUpdateBlockNumberStateMachine<Skipped>),
}

#[serde_as]
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Default)]
struct CheckVersion {
    block: Version,
}

#[serde_as]
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Default)]
struct Version {
    #[serde_as(as = "DisplayFromStr")]
    starknet_version: StarknetVersion,
}

impl StateMachine for StateUpdateBlockNumberStateMachineWrapper {
    fn run(
        &mut self,
        request_body: String,
        response_body: String,
        path: String,
    ) -> StateMachineResult {
        if self.filter(path) {
            self.step(request_body, response_body)
        } else {
            *self = StateUpdateBlockNumberStateMachineWrapper::Skipped(match self {
                StateUpdateBlockNumberStateMachineWrapper::Ok(machine) => machine.to_skipped(),
                StateUpdateBlockNumberStateMachineWrapper::Invalid(machine) => {
                    machine.clone().to_skipped()
                }
                StateUpdateBlockNumberStateMachineWrapper::Skipped(machine) => machine.clone(),
            });
            StateMachineResult::Skipped
        }
    }

    fn filter(&self, path: String) -> bool {
        fn validate_regex(input_path: String, machine_path: String) -> bool {
            if input_path.starts_with(&machine_path) {
                let re = Regex::new(
                    r"^/feeder_gateway/get_state_update\?blockNumber=\d+&includeBlock=true$",
                )
                .unwrap();
                re.is_match(input_path.as_str())
            } else {
                false
            }
        }

        match self {
            StateUpdateBlockNumberStateMachineWrapper::Ok(machine) => {
                validate_regex(path, machine.path.clone())
            }
            StateUpdateBlockNumberStateMachineWrapper::Invalid(machine) => {
                validate_regex(path, machine.path.clone())
            }
            StateUpdateBlockNumberStateMachineWrapper::Skipped(machine) => {
                validate_regex(path, machine.path.clone())
            }
        }
    }

    fn step(&mut self, _request_body: String, response_body: String) -> StateMachineResult {
        *self = match self {
            StateUpdateBlockNumberStateMachineWrapper::Ok(machine) => {
                match serde_json::from_str::<CheckVersion>(&response_body.clone()) {
                    JsonResult::Ok(version) => {
                        let pre_v_0_13_2 =
                            version.block.starknet_version < StarknetVersion::V_0_13_2;

                        if pre_v_0_13_2 {
                            StateUpdateBlockNumberStateMachineWrapper::Skipped(
                                machine.clone().to_skipped(),
                            )
                        } else {
                            match serde_json::from_str::<BlockStateUpdate>(&response_body) {
                                JsonResult::Ok(block_state_machine) => {
                                    let valid_receipt_commitment = match compute_receipt_commitment(
                                        block_state_machine.clone().block.transaction_receipts,
                                    ) {
                                        std::result::Result::Ok(commitment) => {
                                            commitment
                                                == block_state_machine.block.receipt_commitment
                                        }
                                        Err(_) => false,
                                    };

                                    let valid_transaction_commitment =
                                        match calculate_transaction_commitment(
                                            &block_state_machine.clone().block.transactions,
                                        ) {
                                            std::result::Result::Ok(commitment) => {
                                                commitment
                                                    == block_state_machine
                                                        .block
                                                        .transaction_commitment
                                            }
                                            Err(_) => false,
                                        };

                                    let valid_event_commitment = match compute_event_commitment(
                                        block_state_machine.clone().block.transaction_receipts,
                                    ) {
                                        std::result::Result::Ok(commitment) => {
                                            commitment == block_state_machine.block.event_commitment
                                        }
                                        Err(_) => false,
                                    };

                                    let valid_state_diff_commitment =
                                        compute_state_diff(
                                            block_state_machine.clone().state_update,
                                        ) == block_state_machine.block.state_diff_commitment;

                                    let is_valid_block_hash = match compute_final_hash(
                                        &block_state_machine.block.clone().into(),
                                    ) {
                                        std::result::Result::Ok(block_hash) => {
                                            block_hash == block_state_machine.block.block_hash
                                        }
                                        Err(_) => false,
                                    };

                                    let valid_hashes = valid_receipt_commitment
                                        && valid_state_diff_commitment
                                        && valid_event_commitment
                                        && valid_transaction_commitment
                                        && is_valid_block_hash;

                                    if valid_hashes {
                                        StateUpdateBlockNumberStateMachineWrapper::Ok(
                                            machine.clone(),
                                        )
                                    } else {
                                        StateUpdateBlockNumberStateMachineWrapper::Invalid(
                                            machine.clone().to_invalid(),
                                        )
                                    }
                                }
                                Err(_) => StateUpdateBlockNumberStateMachineWrapper::Invalid(
                                    machine.clone().to_invalid(),
                                ),
                            }
                        }
                    }
                    Err(_) => StateUpdateBlockNumberStateMachineWrapper::Invalid(
                        machine.clone().to_invalid(),
                    ),
                }
            }

            StateUpdateBlockNumberStateMachineWrapper::Invalid(machine) => {
                StateUpdateBlockNumberStateMachineWrapper::Invalid(machine.clone())
            }
            StateUpdateBlockNumberStateMachineWrapper::Skipped(machine) => {
                StateUpdateBlockNumberStateMachineWrapper::Skipped(machine.clone())
            }
        };

        match self {
            StateUpdateBlockNumberStateMachineWrapper::Ok(_) => StateMachineResult::Ok(
                "State Update by block number request SUCCESSFUL".to_string(),
            ),
            StateUpdateBlockNumberStateMachineWrapper::Invalid(_) => StateMachineResult::Invalid(
                "State Update by block number request FAILED".to_string(),
            ),
            StateUpdateBlockNumberStateMachineWrapper::Skipped(_) => StateMachineResult::Skipped,
        }
    }
}

pub fn compute_state_diff(state_update_gateway: GatewayStateUpdate) -> Felt {
    let state_update_common: StateUpdate = state_update_gateway.into();
    compute(
        &state_update_common.contract_updates,
        &state_update_common.system_contract_updates,
        &state_update_common.declared_cairo_classes,
        &state_update_common.declared_sierra_classes,
    )
}

pub fn compute_receipt_commitment(
    transaction_receipts: Vec<(Receipt, Vec<Event>)>,
) -> Result<Felt> {
    let receipts: Vec<Receipt> = transaction_receipts
        .into_iter()
        .map(|(receipt, _events)| receipt)
        .collect();

    calculate_receipt_commitment(&receipts)
}

pub fn compute_event_commitment(transaction_receipts: Vec<(Receipt, Vec<Event>)>) -> Result<Felt> {
    let transaction_events: Vec<(TransactionHash, Vec<Event>)> = transaction_receipts
        .into_iter()
        .map(|(receipt, events)| (receipt.transaction_hash, events.to_owned()))
        .collect();

    calculate_event_commitment(&transaction_events)
}

impl StateUpdateBlockNumberStateMachineWrapper {
    pub fn new() -> Self {
        StateUpdateBlockNumberStateMachineWrapper::Ok(StateUpdateBlockNumberStateMachine::new())
    }
}

impl Default for StateUpdateBlockNumberStateMachineWrapper {
    fn default() -> Self {
        Self::new()
    }
}
