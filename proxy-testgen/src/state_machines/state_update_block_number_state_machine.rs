use crate::StateMachine;
use crate::StateMachineResult;
use production_nodes_types::pathfinder_types::types::alpha_sepolia_blocks::BlockStateMachine;
use production_nodes_types::pathfinder_types::types::block_hash::compute_final_hash;
use regex::Regex;
use serde_json::Result as JsonResult;

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
                match serde_json::from_str::<BlockStateMachine>(&response_body) {
                    JsonResult::Ok(block_state_machine) => {
                        let is_valid_hash =
                            match compute_final_hash(&block_state_machine.block.clone().into()) {
                                std::result::Result::Ok(block_hash) => {
                                    block_hash == block_state_machine.block.block_hash
                                }
                                Err(_) => false,
                            };

                        if is_valid_hash {
                            StateUpdateBlockNumberStateMachineWrapper::Ok(machine.clone())
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
            StateUpdateBlockNumberStateMachineWrapper::Invalid(_) => {
                StateMachineResult::Invalid("State Update by block number request FAILED".to_string())
            }
            StateUpdateBlockNumberStateMachineWrapper::Skipped(_) => StateMachineResult::Skipped,
        }
    }
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
