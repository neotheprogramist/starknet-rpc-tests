use reqwest::header::CONTENT_TYPE;
use serde_json::Value;
use std::collections::HashMap;
pub async fn starknet_chain_id(rpc_url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut map = HashMap::new();
    map.insert("jsonrpc", "2.0");
    map.insert("id", "1");
    map.insert("method", "starknet_chainId");
    let client = reqwest::Client::new();
    let res = client
        .post(rpc_url)
        .header(CONTENT_TYPE, "application/json")
        .json(&map)
        .send()
        .await?;
    let contents = res.text().await?;
    let from_str: Value = serde_json::from_str(&contents)?;
    let chain_id = from_str["result"]
        .as_str()
        .ok_or("Chaind ID not found in JSON response")?;

    Ok(chain_id.to_string())
}
