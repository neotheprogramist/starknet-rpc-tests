mod starknet_add_declare_transaction;
mod starknet_block_number;
mod starknet_chain_id;
mod starknet_estimate_fee;
mod starknet_get_nonce;
use crate::starknet_block_number::starknet_block_number;
use starknet_chain_id::starknet_chain_id;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url = "https://starknet-sepolia.public.blastapi.io/rpc/v0_7";

    let chain_id: String = starknet_chain_id(rpc_url).await?;
    dbg!(chain_id);

    let block_number: String = starknet_block_number(rpc_url).await?;
    dbg!(block_number);

    // let contract_address = "";
    // let chain_id: String = starknet_estimate_fee(rpc_url).await?;

    Ok(())
}
