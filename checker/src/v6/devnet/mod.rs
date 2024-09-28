pub mod endpoints;
pub mod errors;
pub mod helpers;
pub mod models;

use colored::*;
use errors::DevnetError;
use helpers::prepare_postman_send_message_to_l2;
use models::{
    AccountBalanceParams, AccountBalanceResponse, DumpPath, LoadPath, MsgToL2,
    PostmanFlushParameters, PostmanFlushResponse, PostmanLoadL1MessagingContractParams,
    PostmanLoadL1MessagingContractResponse, PostmanSendMessageToL2Response, SerializableAccount,
};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::v0_6_0::PriceUnit;

use std::future::Future;
use tracing::{error, info};
use url::Url;

pub struct Devnet {
    pub url: Url,
    pub network_url: Url,
}

impl Devnet {
    pub fn new(url: Url, network_url: Url) -> Result<Self, DevnetError> {
        Ok(Self { url, network_url })
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
    fn postman_load_l1_messaging_contract(
        &self,
        params: PostmanLoadL1MessagingContractParams,
    ) -> impl Future<Output = Result<PostmanLoadL1MessagingContractResponse, DevnetError>> + Send;
    fn postman_flush(
        &self,
        params: PostmanFlushParameters,
    ) -> impl Future<Output = Result<PostmanFlushResponse, DevnetError>> + Send;
    fn postman_send_message_to_l2(
        &self,
        params: MsgToL2,
    ) -> impl Future<Output = Result<PostmanSendMessageToL2Response, DevnetError>> + Send;
    fn restart(&self) -> impl Future<Output = Result<(), DevnetError>> + Send;
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
    async fn postman_load_l1_messaging_contract(
        &self,
        params: PostmanLoadL1MessagingContractParams,
    ) -> Result<PostmanLoadL1MessagingContractResponse, DevnetError> {
        endpoints::postman_load_l1_messaging_contract(self.url.clone(), params).await
    }
    async fn postman_flush(
        &self,
        params: PostmanFlushParameters,
    ) -> Result<PostmanFlushResponse, DevnetError> {
        endpoints::postman_flush(self.url.clone(), params).await
    }
    async fn postman_send_message_to_l2(
        &self,
        params: MsgToL2,
    ) -> Result<PostmanSendMessageToL2Response, DevnetError> {
        endpoints::postman_send_message_to_l2(self.url.clone(), params).await
    }
    async fn restart(&self) -> Result<(), DevnetError> {
        endpoints::restart(self.url.clone()).await
    }
}

pub async fn test_devnet_endpoints(
    url: Url,
    network_url: Url,
    sierra_path: &str,
    casm_path: &str,
) -> Result<(), DevnetError> {
    info!("{}", "⌛ Testing Devnet V6 endpoints -- START ⌛".yellow());

    let devnet = Devnet::new(url.clone(), network_url)?;

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

    match devnet
        .postman_load_l1_messaging_contract(PostmanLoadL1MessagingContractParams {
            network_url: devnet.network_url.to_string(),
            address: Some("0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".to_string()),
        })
        .await
    {
        Ok(_) => {
            info!(
                "{} {}",
                "✓ Devnet postman_load_l1_messaging_contract COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Devnet postman_load_l1_messaging_contract INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    match devnet
        .postman_flush(PostmanFlushParameters { dry_run: false })
        .await
    {
        Ok(_) => {
            info!(
                "{} {}",
                "✓ Devnet postman_flush COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Devnet postman_flush INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    let msg_to_l2 = prepare_postman_send_message_to_l2(
        url.clone(),
        sierra_path,
        casm_path,
        devnet.network_url.clone(),
    )
    .await
    .unwrap();

    match devnet.postman_send_message_to_l2(msg_to_l2.clone()).await {
        Ok(_) => {
            info!(
                "{} {}",
                "✓ Devnet postman_send_message_to_l2 COMPATIBLE".green(),
                "✓".green()
            );
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Devnet postman_send_message_to_l2 INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    info!("{}", "🏁 Testing Devnet V6 endpoints -- END 🏁".yellow());
    Ok(())
}
