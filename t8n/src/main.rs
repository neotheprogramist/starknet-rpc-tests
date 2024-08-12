pub mod starknet;
use starknet::state::{starknet_config::StarknetConfig, Starknet};
use tracing::info;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let result: Starknet = Starknet::new(&StarknetConfig::default()).unwrap();
    for (index, acc) in result.predeployed_accounts.accounts.iter().enumerate() {
        info!(
            "Acc {} {:?} {:?}",
            index, acc.account_address, acc.initial_balance
        );
    }
}
