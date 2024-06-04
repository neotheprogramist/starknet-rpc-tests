use reqwest::header::{HeaderMap, ACCEPT, CONTENT_TYPE};
use serde_json::Value;
use std::collections::HashMap;
pub async fn starknet_estimate_fee(rpc_url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut map = HashMap::new();
    map.insert("jsonrpc", serde_json::Value::from("2.0"));
    map.insert("id", serde_json::Value::from("1"));
    map.insert("method", serde_json::Value::from("starknet_estimateFee"));

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert(ACCEPT, "application/json".parse().unwrap());

    let mut params = Vec::new();
    params.push(serde_json::Value::from("latest"));
    map.insert("params", serde_json::Value::from(params));

    let client = reqwest::Client::new();
    let res = client
        .post(rpc_url)
        .headers(headers)
        .json(&map)
        .send()
        .await?;
    let contents = res.text().await?;
    println!("{}", contents);

    let from_str: Value = serde_json::from_str(&contents)?;
    println!("{}", from_str);
    let chain_id = from_str["result"]
        .as_str()
        .ok_or("Chaind ID not found in JSON response")?;

    Ok(chain_id.to_string())
}
