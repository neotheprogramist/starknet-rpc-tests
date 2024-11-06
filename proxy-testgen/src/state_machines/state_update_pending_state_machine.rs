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
pub struct StateUpdatePendingStateMachine<S> {
    path: String,
    state: S,
}

impl StateUpdatePendingStateMachine<Ok> {
    pub fn new() -> Self {
        Self {
            path: "/feeder_gateway/get_state_update?blockNumber=pending&includeBlock=true"
                .to_string(),
            state: Ok,
        }
    }

    pub fn to_invalid(self) -> StateUpdatePendingStateMachine<Invalid> {
        StateUpdatePendingStateMachine {
            path: self.path,
            state: Invalid,
        }
    }

    pub fn to_skipped(&self) -> StateUpdatePendingStateMachine<Skipped> {
        StateUpdatePendingStateMachine {
            path: self.path.clone(),
            state: Skipped,
        }
    }
}

impl Default for StateUpdatePendingStateMachine<Ok> {
    fn default() -> Self {
        Self::new()
    }
}

impl StateUpdatePendingStateMachine<Invalid> {
    pub fn to_skipped(self) -> StateUpdatePendingStateMachine<Skipped> {
        StateUpdatePendingStateMachine {
            path: self.path,
            state: Skipped,
        }
    }
}

impl StateUpdatePendingStateMachine<Skipped> {
    pub fn to_okk(self) -> StateUpdatePendingStateMachine<Ok> {
        StateUpdatePendingStateMachine {
            path: self.path,
            state: Ok,
        }
    }
}

pub enum StateUpdatePendingStateMachineWrapper {
    Ok(StateUpdatePendingStateMachine<Ok>),
    Invalid(StateUpdatePendingStateMachine<Invalid>),
    Skipped(StateUpdatePendingStateMachine<Skipped>),
}

impl StateMachine for StateUpdatePendingStateMachineWrapper {
    fn run(
        &mut self,
        request_body: String,
        response_body: String,
        path: String,
    ) -> StateMachineResult {
        if self.filter(path) {
            self.step(request_body, response_body)
        } else {
            *self = StateUpdatePendingStateMachineWrapper::Skipped(match self {
                StateUpdatePendingStateMachineWrapper::Ok(machine) => machine.to_skipped(),
                StateUpdatePendingStateMachineWrapper::Invalid(machine) => {
                    machine.clone().to_skipped()
                }
                StateUpdatePendingStateMachineWrapper::Skipped(machine) => machine.clone(),
            });
            StateMachineResult::Skipped
        }
    }

    fn filter(&self, path: String) -> bool {
        match self {
            StateUpdatePendingStateMachineWrapper::Ok(machine) => machine.path == path,
            StateUpdatePendingStateMachineWrapper::Invalid(machine) => machine.path == path,
            StateUpdatePendingStateMachineWrapper::Skipped(machine) => machine.path == path,
        }
    }

    fn step(&mut self, request_body: String, response_body: String) -> StateMachineResult {
        *self = match self {
            //TODO: Response Validation
            StateUpdatePendingStateMachineWrapper::Ok(machine) => {
                StateUpdatePendingStateMachineWrapper::Ok(machine.clone())
            }
            StateUpdatePendingStateMachineWrapper::Invalid(machine) => {
                StateUpdatePendingStateMachineWrapper::Invalid(machine.clone())
            }
            StateUpdatePendingStateMachineWrapper::Skipped(machine) => {
                StateUpdatePendingStateMachineWrapper::Skipped(machine.clone())
            }
        };

        match self {
            StateUpdatePendingStateMachineWrapper::Ok(_) => {
                StateMachineResult::Ok("State Update request SUCCESSFUL".to_string())
            }
            StateUpdatePendingStateMachineWrapper::Invalid(_) => {
                StateMachineResult::Invalid("State Update request FAILED".to_string())
            }
            StateUpdatePendingStateMachineWrapper::Skipped(_) => StateMachineResult::Skipped,
        }
    }
}

impl StateUpdatePendingStateMachineWrapper {
    pub fn new() -> Self {
        StateUpdatePendingStateMachineWrapper::Ok(StateUpdatePendingStateMachine::new())
    }
}

impl Default for StateUpdatePendingStateMachineWrapper {
    fn default() -> Self {
        Self::new()
    }
}
