use reqwest::header::{HeaderMap, ACCEPT, CONTENT_TYPE};
use serde_json::{json, Value};
use std::collections::HashMap;

pub async fn call(
    rpc_url: &str,
    method: &str,
    params: Vec<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut map: HashMap<&str, serde_json::Value> = HashMap::new();
    map.insert("jsonrpc", serde_json::Value::from("2.0"));
    map.insert("id", serde_json::Value::from("1"));
    map.insert("method", serde_json::Value::from(method));
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert(ACCEPT, "application/json".parse().unwrap());
    if !params.is_empty() {
        let params_stored: Vec<Value> = params.iter().map(|param| json!(param)).collect();
        map.insert("params", serde_json::Value::Array(params_stored.clone()));
        println!("JSON Parameters: {:?}", params_stored.clone());
    }

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
    let result = from_str["result"].to_string();

    Ok(result.to_string())
}
