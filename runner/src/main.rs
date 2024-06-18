mod args;

use args::Args;
use clap::Parser;
use colored::Colorize;
use shared::{account_balance::Version, clients::devnet_client::DevnetClient};
use starknet_crypto::FieldElement;
use starknet_signers::{LocalWallet, SigningKey};
use tracing::info;
use url::Url;
use utils::{
    account::{
        single_owner::{ExecutionEncoding, SingleOwnerAccount},
        ConnectedAccount,
    },
    provider::Provider,
    transports::http::HttpTransport,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let args = Args::parse();

    // let account_balance_params = AccountBalanceParams {
    //     address: args.account_address,
    //     unit: "WEI".to_string(),
    //     block_tag: "latest".to_string(),
    // };
    // account_balance(&account_balance_params, &args.vers, args.url).await?;

    let signer: LocalWallet = LocalWallet::from(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be("0xe1406455b7d66b1690803be066cbe5e").unwrap(),
    ));
    let address = FieldElement::from_hex_be(
        "0x78662e7352d062084b0010068b99288486c2d8b914f6e2a55ce945f8792c8b1",
    )
    .unwrap();
    let chain_id = FieldElement::from_hex_be("0x534e5f5345504f4c4941").unwrap();
    let encoding = ExecutionEncoding::New;
    let account: SingleOwnerAccount<DevnetClient<HttpTransport>, LocalWallet>;

    match &args.vers {
        Version::V0_0_5 => {
            let devnet_v5_url = Url::parse("http://localhost:5051")?;
            let client = DevnetClient::new(HttpTransport::new(devnet_v5_url));
            account = SingleOwnerAccount::new(client, signer, address, chain_id, encoding);
        }
        Version::V0_0_6 => {
            let devnet_v6_url = Url::parse("http://localhost:5050")?;
            let client = DevnetClient::new(HttpTransport::new(devnet_v6_url));
            account = SingleOwnerAccount::new(client, signer, address, chain_id, encoding);
        }
    };

    match account.provider().get_config().await {
        Ok(config) => {
            info!("{}", "COMPATIBLE".green());
            println!("{:?}", config);
        }
        Err(_) => info!("{}", "INCOMPATIBLE".red()),
    }

    Ok(())
}
