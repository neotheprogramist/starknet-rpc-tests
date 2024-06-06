mod args;
mod call;
mod errors;
mod transports;
mod utils;
use args::Args;
use clap::Parser;
use errors::RunnerError;
mod tests;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use starknet::{
    core::types::{
        contract::{CompiledClass, SierraClass},
        FieldElement, FlattenedSierraClass,
    },
    signers::SigningKey,
};
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tokio::io::AsyncReadExt;
use transports::{http::HttpTransport, JsonRpcClient};
use url::Url;
use utils::{
    codegen::{
        AddDeclareTransactionRequestRef, BroadcastedDeclareTransactionV3, DataAvailabilityMode,
        GetNonceRequestRef, ResourceBounds, ResourceBoundsMapping,
    },
    BroadcastedDeclareTransaction,
};

fn create_jsonrpc_client() -> JsonRpcClient<HttpTransport> {
    let rpc_url = std::env::var("STARKNET_RPC").unwrap_or("http://localhost:5050/".into());
    JsonRpcClient::new(HttpTransport::new(url::Url::parse(&rpc_url).unwrap()))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();

    // let params = vec![];
    // let chain_id: String = call(rpc_url, "starknet_chainId", params).await?;
    // dbg!(chain_id);

    // let params = vec![];
    // let block_number = call(rpc_url, "starknet_blockNumber", params).await?;
    // dbg!(block_number);

    // let (flattened_sierra_class, compiled_class_hash) = get_compiled_contract(
    //     "target/dev/example_HelloStarknet.contract_class.json",
    //     "target/dev/example_HelloStarknet.compiled_contract_class.json",
    // )
    // .await?;
    // // let contract_class = ContractClass::Sierra(flattened_sierra_class);
    // let secret_key = SigningKey::from_random();
    // let bytes_repr: [u8; 32] = secret_key.secret_scalar().to_bytes_be();
    // let nonce = FieldElement::from_byte_slice_be(&bytes_repr).unwrap();
    // dbg!("HERE");

    // let resource_bounds = ResourceBoundsMapping {
    //     l1_gas: ResourceBounds {
    //         max_amount: 100_000,
    //         max_price_per_unit: 5,
    //     },
    //     l2_gas: ResourceBounds {
    //         max_amount: 0,
    //         max_price_per_unit: 0,
    //     },
    // };

    // let txn: BroadcastedDeclareTransactionV3 = BroadcastedDeclareTransactionV3 {
    //     sender_address: args.sender_address,
    //     compiled_class_hash,
    //     signature: vec![FieldElement::from_hex_be(
    //         "0x560113f0558053f055f1139055805390515117505580574055e1174055805",
    //     )?],
    //     nonce,
    //     contract_class: Arc::new(flattened_sierra_class),
    //     resource_bounds,
    //     is_query: false,
    //     paymaster_data: Vec::new(),
    //     account_deployment_data: Vec::new(),
    //     tip: 0,
    //     nonce_data_availability_mode: DataAvailabilityMode::L2,
    //     fee_data_availability_mode: DataAvailabilityMode::L2,
    // };
    // let declared_transaction = BroadcastedDeclareTransaction::V3(txn);
    // let params = AddDeclareTransactionRequestRef {
    //     declare_transaction: declared_transaction.as_ref(),
    // };

    // dbg!(send_post_request(rpc_url, &params, "starknet_addDeclareTransaction").await?);

    // let params = GetNonceRequestRef {
    //     block_id: block_id.as_ref(),
    //     contract_address: contract_address.as_ref(),
    // };

    // dbg!(send_post_request(rpc_url, &params, "starknet_getNonce").await?);

    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcResponse<T> {
    Success { id: u64, result: T },
    Error { id: u64, error: JsonRpcError },
}
#[derive(Debug, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
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

async fn send_post_request<P, R>(url: Url, body: &P, method: &str) -> Result<R, RunnerError>
where
    P: Serialize + Send + Sync,
    R: DeserializeOwned,
{
    let request_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "1",
        "method": method,
        "params": body, // Include the body parameter here
    });
    let client = reqwest::Client::new();
    let mut map: HashMap<&str, serde_json::Value> = HashMap::new();
    map.insert("jsonrpc", serde_json::Value::from("2.0"));
    map.insert("id", serde_json::Value::from("1"));
    map.insert(
        "method",
        serde_json::Value::from("starknet_addDeclareTransaction"),
    );

    dbg!(
        "Sending POST request to sequencer API ({}): {}",
        url.clone(),
        request_body.clone()
    );

    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&map)
        .body(serde_json::to_string(&request_body).unwrap())
        .send()
        .await?;

    dbg!("Response from sequencer API: {}", res.status());

    let body = res.text().await?;

    dbg!("Response from sequencer API: {}", body.clone());

    let result = serde_json::from_str(&body)?;
    Ok(result)
}
