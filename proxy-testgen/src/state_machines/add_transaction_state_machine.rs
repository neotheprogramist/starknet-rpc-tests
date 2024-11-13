use crate::StateMachine;
use crate::StateMachineResult;
use num_traits::Num;
use production_nodes_types::pathfinder_types::types::block_builder_input::TransactionHash;
use production_nodes_types::pathfinder_types::types::gateway_add_transaction::*;
use production_nodes_types::pathfinder_types::types::gateway_state_update::StarknetVersion;
use serde::Deserialize;
use serde_json::Result as JsonResult;
use serde_with::{serde_as, DisplayFromStr};

#[derive(Clone)]
pub struct Ok;

#[derive(Clone)]
pub struct Invalid;

#[derive(Clone)]
pub struct Skipped;

#[derive(Clone)]
pub struct AddTransactionStateMachine<S> {
    path: String,
    transaction_type: Option<AddTransactionResponseType>,
    state: S,
}

impl<S> AddTransactionStateMachine<S> {
    pub fn get_state(&self) -> &S {
        &self.state
    }
}

impl AddTransactionStateMachine<Ok> {
    pub fn new() -> Self {
        Self {
            path: "/gateway/add_transaction".to_string(),
            state: Ok,
            transaction_type: None,
        }
    }

    pub fn to_invalid(self) -> AddTransactionStateMachine<Invalid> {
        AddTransactionStateMachine {
            path: self.path,
            state: Invalid,
            transaction_type: self.transaction_type,
        }
    }

    pub fn to_skipped(&self) -> AddTransactionStateMachine<Skipped> {
        AddTransactionStateMachine {
            path: self.path.clone(),
            state: Skipped,
            transaction_type: None,
        }
    }
}

impl Default for AddTransactionStateMachine<Ok> {
    fn default() -> Self {
        Self::new()
    }
}

impl AddTransactionStateMachine<Invalid> {
    pub fn to_skipped(self) -> AddTransactionStateMachine<Skipped> {
        AddTransactionStateMachine {
            path: self.path,
            state: Skipped,
            transaction_type: None,
        }
    }
}

pub enum AddTransactionStateMachineWrapper {
    Ok(AddTransactionStateMachine<Ok>),
    Invalid(AddTransactionStateMachine<Invalid>),
    Skipped(AddTransactionStateMachine<Skipped>),
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

fn validate_transaction_hash(transaction_hash: TransactionHash) -> bool {
    let transaction_hash = transaction_hash
        .to_hex_string()
        .trim_start_matches("0x")
        .to_string();

    match (
        num_bigint::BigUint::from_str_radix(&transaction_hash, 16),
        num_bigint::BigUint::from_str_radix(
            "800000000000011000000000000000000000000000000000000000000000001",
            16,
        ),
    ) {
        (std::result::Result::Ok(transaction_hash_int), std::result::Result::Ok(stark_prime)) => {
            transaction_hash_int < stark_prime
        }
        _ => false,
    }
}

impl StateMachine for AddTransactionStateMachineWrapper {
    fn run(
        &mut self,
        request_body: String,
        response_body: String,
        path: String,
    ) -> StateMachineResult {
        if self.filter(path) {
            self.step(request_body, response_body)
        } else {
            *self = AddTransactionStateMachineWrapper::Skipped(match self {
                AddTransactionStateMachineWrapper::Ok(machine) => machine.to_skipped(),
                AddTransactionStateMachineWrapper::Invalid(machine) => machine.clone().to_skipped(),
                AddTransactionStateMachineWrapper::Skipped(machine) => machine.clone(),
            });
            StateMachineResult::Skipped
        }
    }

    fn filter(&self, path: String) -> bool {
        match self {
            AddTransactionStateMachineWrapper::Ok(machine) => machine.path == path,
            AddTransactionStateMachineWrapper::Invalid(machine) => machine.path == path,
            AddTransactionStateMachineWrapper::Skipped(machine) => machine.path == path,
        }
    }
    fn step(&mut self, request_body: String, response_body: String) -> StateMachineResult {
        *self = match self {
            AddTransactionStateMachineWrapper::Ok(machine) => {
                match serde_json::from_str::<AddTransactionResponseType>(&request_body) {
                    JsonResult::Ok(version) => {
                        machine.transaction_type = Some(version);
                        AddTransactionStateMachineWrapper::Ok(machine.clone())
                    }
                    Err(_) => {
                        AddTransactionStateMachineWrapper::Invalid(machine.clone().to_invalid())
                    }
                }
            }
            AddTransactionStateMachineWrapper::Invalid(machine) => {
                AddTransactionStateMachineWrapper::Invalid(machine.clone())
            }
            AddTransactionStateMachineWrapper::Skipped(machine) => {
                AddTransactionStateMachineWrapper::Skipped(machine.clone())
            }
        };

        *self = match self {
            AddTransactionStateMachineWrapper::Ok(machine) => {
                match machine.transaction_type.clone() {
                    Some(AddTransactionResponseType::Invoke(_)) => {
                        match serde_json::from_str::<InvokeResponse>(&response_body) {
                            JsonResult::Ok(invoke_response) => {
                                if validate_transaction_hash(invoke_response.transaction_hash) {
                                    machine.transaction_type =
                                        Some(AddTransactionResponseType::Invoke(invoke_response));
                                    AddTransactionStateMachineWrapper::Ok(machine.clone())
                                } else {
                                    machine.transaction_type = None;
                                    AddTransactionStateMachineWrapper::Ok(machine.clone())
                                }
                            }
                            Err(_) => AddTransactionStateMachineWrapper::Invalid(
                                machine.clone().to_invalid(),
                            ),
                        }
                    }
                    Some(AddTransactionResponseType::Declare(_)) => {
                        match serde_json::from_str::<DeclareResponse>(&response_body) {
                            JsonResult::Ok(delcare_response) => {
                                if validate_transaction_hash(delcare_response.transaction_hash) {
                                    machine.transaction_type =
                                        Some(AddTransactionResponseType::Declare(delcare_response));

                                    AddTransactionStateMachineWrapper::Ok(machine.clone())
                                } else {
                                    machine.transaction_type = None;
                                    AddTransactionStateMachineWrapper::Ok(machine.clone())
                                }
                            }
                            Err(_) => AddTransactionStateMachineWrapper::Invalid(
                                machine.clone().to_invalid(),
                            ),
                        }
                    }
                    Some(AddTransactionResponseType::DeployAccount(_)) => {
                        match serde_json::from_str::<DeployAccountResponse>(&response_body) {
                            JsonResult::Ok(deploy_acc_response) => {
                                if validate_transaction_hash(deploy_acc_response.transaction_hash) {
                                    machine.transaction_type =
                                        Some(AddTransactionResponseType::DeployAccount(
                                            deploy_acc_response,
                                        ));
                                    AddTransactionStateMachineWrapper::Ok(machine.clone())
                                } else {
                                    machine.transaction_type = None;
                                    AddTransactionStateMachineWrapper::Ok(machine.clone())
                                }
                            }
                            Err(_) => AddTransactionStateMachineWrapper::Invalid(
                                machine.clone().to_invalid(),
                            ),
                        }
                    }
                    None => {
                        AddTransactionStateMachineWrapper::Invalid(machine.clone().to_invalid())
                    }
                }
            }
            AddTransactionStateMachineWrapper::Invalid(machine) => {
                AddTransactionStateMachineWrapper::Invalid(machine.clone())
            }
            AddTransactionStateMachineWrapper::Skipped(machine) => {
                AddTransactionStateMachineWrapper::Skipped(machine.clone())
            }
        };

        match self {
            AddTransactionStateMachineWrapper::Ok(_) => {
                let message = self.get_message();
                StateMachineResult::Ok(message)
            }
            AddTransactionStateMachineWrapper::Invalid(_) => {
                let message = self.get_message();
                StateMachineResult::Invalid(message)
            }
            AddTransactionStateMachineWrapper::Skipped(_) => StateMachineResult::Skipped,
        }
    }
}

impl AddTransactionStateMachineWrapper {
    pub fn new() -> Self {
        AddTransactionStateMachineWrapper::Ok(AddTransactionStateMachine::new())
    }

    pub fn get_message(&self) -> String {
        match self {
            AddTransactionStateMachineWrapper::Ok(machine) => {
                machine.transaction_type.clone().unwrap().to_string()
            }
            AddTransactionStateMachineWrapper::Invalid(machine) => {
                machine.transaction_type.clone().unwrap().to_string()
            }
            AddTransactionStateMachineWrapper::Skipped(_) => "".to_string(),
        }
    }
}

impl Default for AddTransactionStateMachineWrapper {
    fn default() -> Self {
        Self::new()
    }
}
