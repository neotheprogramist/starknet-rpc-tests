use axum::response;
use regex::Regex;
use reqwest::{Body, Method, Response, Url};
use rustls::ServerConfig;
use starknet_types_core::felt::Felt;
use starknet_types_rpc::v0_7_1::BlockHashAndNumber;
use std::{net::TcpStream, sync::Arc};

pub const GET_BLOCK_NUMBER_URL: &str =
    "https://alpha-sepolia.starknet.io/feeder_gateway/get_block?blockNumber=latest&headerOnly=true";

pub trait TestCase {
    fn run(&mut self, url: Url);
}

#[derive(Debug, Clone)]
pub struct WaitForLatestBlockNumberRequest;

#[derive(Debug)]
pub struct WaitForNumberAndHashResponse;

#[derive(Debug)]
pub struct Done {
    response_body: String,
}

#[derive(Debug, Clone)]
pub struct InvalidHash;

#[derive(Debug, Clone)]
pub struct BlockNumberStateMachine<S> {
    state: S,
}

impl BlockNumberStateMachine<WaitForLatestBlockNumberRequest> {
    pub fn new() -> Self {
        BlockNumberStateMachine {
            state: WaitForLatestBlockNumberRequest,
        }
    }

    pub fn transition(self) -> BlockNumberStateMachineWrapper {
        BlockNumberStateMachineWrapper::WaitForNumberAndHashResponse(BlockNumberStateMachine {
            state: WaitForNumberAndHashResponse,
        })
    }
}

impl BlockNumberStateMachine<WaitForNumberAndHashResponse> {
    pub fn transition(
        self,
        response_body: String,
    ) -> Result<BlockNumberStateMachineWrapper, serde_json::Error> {
        let deserialized_response: BlockHashAndNumber<Felt> = serde_json::from_str(&response_body)?;

        let block_hash_str = deserialized_response.block_hash.to_hex_string();

        let is_valid_hash = Regex::new(r"^0x[0-9a-fA-F]{64}$")
            .map(|re| re.is_match(&block_hash_str))
            .unwrap_or(false);

        if is_valid_hash {
            Ok(BlockNumberStateMachineWrapper::Done(
                BlockNumberStateMachine {
                    state: Done { response_body },
                },
            ))
        } else {
            Ok(BlockNumberStateMachineWrapper::InvalidHash(
                BlockNumberStateMachine { state: InvalidHash },
            ))
        }
    }
}

#[derive(Debug)]
pub enum BlockNumberStateMachineWrapper {
    WaitForLatestBlockNumberRequest(BlockNumberStateMachine<WaitForLatestBlockNumberRequest>),
    WaitForNumberAndHashResponse(BlockNumberStateMachine<WaitForNumberAndHashResponse>),
    Done(BlockNumberStateMachine<Done>),
    InvalidHash(BlockNumberStateMachine<InvalidHash>),
}

impl BlockNumberStateMachineWrapper {
    pub fn step(self, response: Option<String>) -> Result<Self, serde_json::Error> {
        match self {
            BlockNumberStateMachineWrapper::WaitForLatestBlockNumberRequest(machine) => {
                Ok(machine.transition())
            }
            BlockNumberStateMachineWrapper::WaitForNumberAndHashResponse(machine) => {
                if let Some(resp) = response {
                    machine.transition(resp)
                } else {
                    panic!("Expected response for transition from WaitForLatestBlockNumberRequest");
                }
            }
            BlockNumberStateMachineWrapper::Done(_) => {
                println!("Proxy validation test succesfull");
                Ok(
                    BlockNumberStateMachineWrapper::WaitForLatestBlockNumberRequest(
                        BlockNumberStateMachine::new(),
                    ),
                )
            }
            BlockNumberStateMachineWrapper::InvalidHash(_) => {
                println!("Proxy validation test error");
                Ok(
                    BlockNumberStateMachineWrapper::WaitForLatestBlockNumberRequest(
                        BlockNumberStateMachine::new(),
                    ),
                )
            }
        }
    }
}

#[derive(Debug)]
pub struct Factory {
    pub block_number_machine: BlockNumberStateMachineWrapper,
}

impl Factory {
    pub fn new() -> Self {
        Factory {
            block_number_machine: BlockNumberStateMachineWrapper::WaitForLatestBlockNumberRequest(
                BlockNumberStateMachine::new(),
            ),
        }
    }
}
