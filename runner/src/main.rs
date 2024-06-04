mod call;

use call::call;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url = "https://starknet-sepolia.public.blastapi.io/rpc/v0_7";

    let params = vec![];
    let chain_id: String = call(rpc_url, "starknet_chainId", params).await?;
    dbg!(chain_id);

    let params = vec![];
    let block_number = call(rpc_url, "starknet_blockNumber", params).await?;
    dbg!(block_number);

    // let params = vec!["latest",contract_address];
    // let block_number = call(rpc_url, "starknet_get_nonce", params).await;
    // dbg!(block_number);

    Ok(())
}
