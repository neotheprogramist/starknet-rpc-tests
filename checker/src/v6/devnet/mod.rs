pub mod endpoints;
pub mod errors;
pub mod models;

use colored::*;
use errors::DevnetError;
use models::{
    AccountBalanceParams, AccountBalanceResponse, DumpPath, LoadPath, SerializableAccount,
};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::v0_6_0::PriceUnit;

use std::future::Future;
use tracing::{error, info};
use url::Url;

pub struct Devnet {
    pub url: Url,
}

impl Devnet {
    pub fn new(url: Url) -> Result<Self, DevnetError> {
        Ok(Self { url })
    }
}

pub trait DevnetEndpoints {
    fn is_alive(&self) -> impl Future<Output = Result<String, DevnetError>> + Send;
    fn predeployed_accounts(
        &self,
    ) -> impl Future<Output = Result<Vec<SerializableAccount>, DevnetError>> + Send;
    fn account_balance(
        &self,
    ) -> impl Future<Output = Result<AccountBalanceResponse, DevnetError>> + Send;
    fn dump(&self, path: DumpPath) -> impl Future<Output = Result<(), DevnetError>> + Send;
    fn load(&self, path: LoadPath) -> impl Future<Output = Result<(), DevnetError>> + Send;
}

impl DevnetEndpoints for Devnet {
    async fn is_alive(&self) -> Result<String, DevnetError> {
        endpoints::is_alive(self.url.clone()).await
    }
    async fn account_balance(&self) -> Result<AccountBalanceResponse, DevnetError> {
        endpoints::get_account_balance(
            self.url.clone(),
            AccountBalanceParams {
                address: Felt::from_hex(
                    "0x64b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691",
                )
                .unwrap(),
                unit: Some(PriceUnit::Wei),
            },
        )
        .await
    }

    async fn predeployed_accounts(&self) -> Result<Vec<SerializableAccount>, DevnetError> {
        endpoints::get_predeployed_accounts(self.url.clone()).await
    }
    async fn dump(&self, path: DumpPath) -> Result<(), DevnetError> {
        endpoints::dump(self.url.clone(), path).await
    }
    async fn load(&self, path: LoadPath) -> Result<(), DevnetError> {
        endpoints::load(self.url.clone(), path).await
    }
}

pub async fn test_devnet_endpoints(url: Url) -> Result<(), DevnetError> {
    info!("{}", "⌛ Testing Devnet V6 endpoints -- START ⌛".yellow());

    let devnet = Devnet::new(url)?;

    match devnet.is_alive().await {
        Ok(_) => {
            info!("{} {}", "✓ Devnet is_alive COMPATIBLE".green(), "✓".green())
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Devnet is_alive INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    match devnet.predeployed_accounts().await {
        Ok(_) => {
            info!(
                "{} {}",
                "✓ Devnet predeployed_accounts COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Devnet predeployed_accounts INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    match devnet.account_balance().await {
        Ok(_) => {
            info!(
                "{} {}",
                "✓ Devnet account_balance COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Devnet account_balance INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    match devnet
        .dump(DumpPath {
            path: Some("./dump".to_string()),
        })
        .await
    {
        Ok(_) => {
            info!("{} {}", "✓ Devnet dump COMPATIBLE".green(), "✓".green())
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Devnet dump INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    match devnet
        .load(LoadPath {
            path: Some("./load".to_string()),
        })
        .await
    {
        Ok(_) => {
            info!("{} {}", "✓ Devnet load COMPATIBLE".green(), "✓".green())
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Devnet load INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    info!("{}", "🏁 Testing Devnet V6 endpoints -- END 🏁".yellow());
    Ok(())
}
