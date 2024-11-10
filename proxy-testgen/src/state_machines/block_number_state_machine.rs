use crate::StateMachine;
use crate::StateMachineResult;
use num_traits::Num;
use starknet_types_rpc::v0_7_1::BlockHashAndNumber;

#[derive(Clone)]
pub struct Ok;

#[derive(Clone)]
pub struct Invalid;

#[derive(Clone)]
pub struct Skipped;

#[derive(Clone)]
pub struct BlockNumberStateMachine<S> {
    path: String,
    state: S,
}

impl<S> BlockNumberStateMachine<S> {
    pub fn get_state(&self) -> &S {
        &self.state
    }
}

impl BlockNumberStateMachine<Ok> {
    pub fn new() -> Self {
        Self {
            path: "/feeder_gateway/get_block?blockNumber=latest&headerOnly=true".to_string(),
            state: Ok,
        }
    }

    pub fn to_invalid(self) -> BlockNumberStateMachine<Invalid> {
        BlockNumberStateMachine {
            path: self.path,
            state: Invalid,
        }
    }

    pub fn to_skipped(&self) -> BlockNumberStateMachine<Skipped> {
        BlockNumberStateMachine {
            path: self.path.clone(),
            state: Skipped,
        }
    }
}

impl Default for BlockNumberStateMachine<Ok> {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockNumberStateMachine<Invalid> {
    pub fn to_skipped(self) -> BlockNumberStateMachine<Skipped> {
        BlockNumberStateMachine {
            path: self.path,
            state: Skipped,
        }
    }
}

impl BlockNumberStateMachine<Skipped> {
    pub fn to_okk(self) -> BlockNumberStateMachine<Ok> {
        BlockNumberStateMachine {
            path: self.path,
            state: Ok,
        }
    }
}

pub enum BlockNumberStateMachineWrapper {
    Ok(BlockNumberStateMachine<Ok>),
    Invalid(BlockNumberStateMachine<Invalid>),
    Skipped(BlockNumberStateMachine<Skipped>),
}

impl StateMachine for BlockNumberStateMachineWrapper {
    fn run(
        &mut self,
        request_body: String,
        response_body: String,
        path: String,
    ) -> StateMachineResult {
        if self.filter(path) {
            self.step(request_body, response_body)
        } else {
            *self = BlockNumberStateMachineWrapper::Skipped(match self {
                BlockNumberStateMachineWrapper::Ok(machine) => machine.to_skipped(),
                BlockNumberStateMachineWrapper::Invalid(machine) => machine.clone().to_skipped(),
                BlockNumberStateMachineWrapper::Skipped(machine) => machine.clone(),
            });
            StateMachineResult::Skipped
        }
    }

    fn filter(&self, path: String) -> bool {
        match self {
            BlockNumberStateMachineWrapper::Ok(machine) => machine.path == path,
            BlockNumberStateMachineWrapper::Invalid(machine) => machine.path == path,
            BlockNumberStateMachineWrapper::Skipped(machine) => machine.path == path,
        }
    }

    fn step(&mut self, _request_body: String, response_body: String) -> StateMachineResult {
        *self = match self {
            BlockNumberStateMachineWrapper::Ok(machine) => {
                let valid_hash =
                    match serde_json::from_str::<BlockHashAndNumber<String>>(&response_body) {
                        std::result::Result::Ok(deserialized_response) => {
                            let block_hash_str = deserialized_response
                                .block_hash
                                .trim_start_matches("0x")
                                .to_string();
                            match (
                            num_bigint::BigUint::from_str_radix(&block_hash_str, 16),
                            num_bigint::BigUint::from_str_radix(
                                "800000000000011000000000000000000000000000000000000000000000001",
                                16,
                            ),
                        ) {
                            (
                                std::result::Result::Ok(block_hash_int),
                                std::result::Result::Ok(stark_prime),
                            ) => block_hash_int < stark_prime,
                            _ => false,
                        }
                        }
                        Err(_) => false,
                    };
                if valid_hash {
                    BlockNumberStateMachineWrapper::Ok(machine.clone())
                } else {
                    BlockNumberStateMachineWrapper::Invalid(machine.clone().to_invalid())
                }
            }
            BlockNumberStateMachineWrapper::Invalid(machine) => {
                BlockNumberStateMachineWrapper::Invalid(machine.clone())
            }
            BlockNumberStateMachineWrapper::Skipped(machine) => {
                BlockNumberStateMachineWrapper::Skipped(machine.clone())
            }
        };

        match self {
            BlockNumberStateMachineWrapper::Ok(_) => {
                StateMachineResult::Ok("Block number request SUCCESSFUL".to_string())
            }
            BlockNumberStateMachineWrapper::Invalid(_) => {
                StateMachineResult::Invalid("Block number request FAILED".to_string())
            }
            BlockNumberStateMachineWrapper::Skipped(_) => StateMachineResult::Skipped,
        }
    }
}

impl BlockNumberStateMachineWrapper {
    pub fn new() -> Self {
        BlockNumberStateMachineWrapper::Ok(BlockNumberStateMachine::new())
    }
}

impl Default for BlockNumberStateMachineWrapper {
    fn default() -> Self {
        Self::new()
    }
}
