pub mod endpoints;
pub mod errors;
pub mod models;

use crate::v5::devnet::models::{
    IncreaseTimeParams, IncreaseTimeResponse, SetTimeParams, SetTimeResponse,
};
use colored::*;
use errors::DevnetError;
use models::{
    AbortBlocksParams, AbortBlocksResponse, AccountBalanceParams, AccountBalanceResponse,
    CreateBlockResponse, ForkStatusResponse, MintTokensParams, MintTokensResponse,
    SerializableAccount,
};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::v0_5_0::FeeUnit;
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
    fn restart(&self) -> impl Future<Output = Result<(), DevnetError>> + Send;
    fn set_time(
        &self,
        params: SetTimeParams,
    ) -> impl Future<Output = Result<SetTimeResponse, DevnetError>> + Send;
    fn increase_time(
        &self,
        params: IncreaseTimeParams,
    ) -> impl Future<Output = Result<IncreaseTimeResponse, DevnetError>> + Send;
    fn mint(
        &self,
        params: MintTokensParams,
    ) -> impl Future<Output = Result<MintTokensResponse, DevnetError>> + Send;
    fn fork_status(&self) -> impl Future<Output = Result<ForkStatusResponse, DevnetError>> + Send;
    fn create_block(&self)
        -> impl Future<Output = Result<CreateBlockResponse, DevnetError>> + Send;
    fn abort_blocks(
        &self,
        params: AbortBlocksParams,
    ) -> impl Future<Output = Result<AbortBlocksResponse, DevnetError>> + Send;
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
                unit: Some(FeeUnit::Wei),
            },
        )
        .await
    }

    async fn predeployed_accounts(&self) -> Result<Vec<SerializableAccount>, DevnetError> {
        endpoints::get_predeployed_accounts(self.url.clone()).await
    }
    async fn restart(&self) -> Result<(), DevnetError> {
        endpoints::restart(self.url.clone()).await
    }
    async fn set_time(&self, params: SetTimeParams) -> Result<SetTimeResponse, DevnetError> {
        endpoints::set_time(self.url.clone(), params).await
    }
    async fn increase_time(
        &self,
        params: IncreaseTimeParams,
    ) -> Result<IncreaseTimeResponse, DevnetError> {
        endpoints::increase_time(self.url.clone(), params).await
    }
    async fn mint(&self, params: MintTokensParams) -> Result<MintTokensResponse, DevnetError> {
        endpoints::mint(self.url.clone(), params).await
    }
    async fn fork_status(&self) -> Result<ForkStatusResponse, DevnetError> {
        endpoints::fork_status(self.url.clone()).await
    }
    async fn create_block(&self) -> Result<CreateBlockResponse, DevnetError> {
        endpoints::create_block(self.url.clone()).await
    }
    async fn abort_blocks(
        &self,
        params: AbortBlocksParams,
    ) -> Result<AbortBlocksResponse, DevnetError> {
        endpoints::abort_blocks(self.url.clone(), params).await
    }
}

pub async fn test_devnet_endpoints(url: Url) -> Result<(), DevnetError> {
    info!("{}", "⌛ Testing Devnet V5 endpoints -- START ⌛".yellow());

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

    match devnet.fork_status().await {
        Ok(_) => {
            info!(
                "{} {}",
                "✓ Devnet fork_status COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Devnet fork_status INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    match devnet
        .set_time(SetTimeParams {
            time: 3203391149,
            generate_block: Some(true),
        })
        .await
    {
        Ok(_) => {
            info!("{} {}", "✓ Devnet set_time COMPATIBLE".green(), "✓".green())
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Devnet set_time INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    match devnet
        .increase_time(IncreaseTimeParams { time: 3735928559 })
        .await
    {
        Ok(_) => {
            info!(
                "{} {}",
                "✓ Devnet increase_time COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Devnet increase_time INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    match devnet
        .mint(MintTokensParams {
            address: Felt::from_hex(
                "0x64b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691",
            )
            .unwrap(),
            amount: 244837814099629,
            unit: Some(FeeUnit::Wei),
        })
        .await
    {
        Ok(_) => {
            info!("{} {}", "✓ Devnet mint COMPATIBLE".green(), "✓".green())
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Devnet mint INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }
    match devnet.create_block().await {
        Ok(_) => {
            info!(
                "{} {}",
                "✓ Devnet create_block COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Devnet create_block INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    match devnet
        .abort_blocks(AbortBlocksParams {
            starting_block_hash: devnet.create_block().await.unwrap().block_hash,
        })
        .await
    {
        Ok(_) => {
            info!(
                "{} {}",
                "✓ Devnet abort_block COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Devnet abort_block INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    match devnet.restart().await {
        Ok(_) => {
            info!("{} {}", "✓ Devnet restart COMPATIBLE".green(), "✓".green())
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Devnet restart INCOMPATIBLE:".red(),
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

    info!("{}", "🏁 Testing Devnet V5 endpoints -- END 🏁".yellow());
    Ok(())
}
