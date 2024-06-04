use reqwest::header::{HeaderMap, ACCEPT, CONTENT_TYPE};
use serde_json::Value;
use std::collections::HashMap;

pub async fn starknet_block_number(rpc_url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut map: HashMap<&str, &str> = HashMap::new();

    map.insert("jsonrpc", "2.0");
    map.insert("id", "1");
    map.insert("method", "starknet_blockNumber");

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert(ACCEPT, "application/json".parse().unwrap());
    let client = reqwest::Client::new();
    let res = client
        .post(rpc_url)
        .headers(headers)
        .json(&map)
        .send()
        .await?;
    // Check for HTTP errors
    if !res.status().is_success() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Request failed with status: {}", res.status()),
        )));
    }
    let contents = res.text().await?;

    let from_str: Value = serde_json::from_str(&contents)?;
    let block_number = from_str["result"]
        .as_i64()
        .ok_or("Block number not found in JSON response")?;
    Ok(block_number.to_string())
}
