mod args;
mod errors;
mod transports;
mod utils;
use args::Args;
use clap::Parser;
use errors::RunnerError;
mod tests;
use starknet::core::types::{
    contract::{CompiledClass, SierraClass},
    FieldElement, FlattenedSierraClass,
};
use tokio::io::AsyncReadExt;
use transports::{http::HttpTransport, JsonRpcClient};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();

    Ok(())
}

fn create_jsonrpc_client() -> JsonRpcClient<HttpTransport> {
    let rpc_url = std::env::var("STARKNET_RPC").unwrap_or("http://localhost:5050/".into());
    JsonRpcClient::new(HttpTransport::new(url::Url::parse(&rpc_url).unwrap()))
}
pub async fn get_compiled_contract(
    sierra_path: &str,
    casm_path: &str,
) -> Result<(FlattenedSierraClass, FieldElement), RunnerError> {
    let mut file = tokio::fs::File::open(sierra_path).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            RunnerError::ReadFileError(
                "Contract json file not found, please execute scarb build command".to_string(),
            )
        } else {
            RunnerError::ReadFileError(e.to_string())
        }
    })?;
    let mut sierra = String::default();
    file.read_to_string(&mut sierra)
        .await
        .map_err(|e| RunnerError::ReadFileError(e.to_string()))?;

    let mut file = tokio::fs::File::open(casm_path).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            RunnerError::ReadFileError(
                "Contract json file not found, please execute scarb build command".to_string(),
            )
        } else {
            RunnerError::ReadFileError(e.to_string())
        }
    })?;
    let mut casm = String::default();
    file.read_to_string(&mut casm)
        .await
        .map_err(|e| RunnerError::ReadFileError(e.to_string()))?;

    let contract_artifact: SierraClass = serde_json::from_str(&sierra)?;
    let compiled_class: CompiledClass = serde_json::from_str(&casm)?;
    let casm_class_hash = compiled_class.class_hash()?;
    let flattened_class = contract_artifact.clone().flatten()?;
    Ok((flattened_class, casm_class_hash))
}
