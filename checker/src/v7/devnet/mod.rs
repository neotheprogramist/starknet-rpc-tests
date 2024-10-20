pub mod endpoints;
pub mod errors;
pub mod helpers;
pub mod models;

use colored::*;
use errors::DevnetError;
use helpers::prepare_postman_send_message_to_l2;
use models::{
    AbortBlocksParams, AbortBlocksResponse, AccountBalanceParams, AccountBalanceResponse,
    CreateBlockResponse, DevnetConfigResponse, DumpPath, IncreaseTimeParams, IncreaseTimeResponse,
    LoadPath, MintTokensParams, MintTokensResponse, MsgToL2, PostmanFlushParameters,
    PostmanFlushResponse, PostmanLoadL1MessagingContractParams,
    PostmanLoadL1MessagingContractResponse, PostmanSendMessageToL2Response, SerializableAccount,
    SetTimeParams, SetTimeResponse,
};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::v0_7_1::PriceUnit;

use std::future::Future;
use tracing::{error, info};
use url::Url;

use crate::v7::rpc::endpoints::utils::restart_devnet;

pub struct Devnet {
    pub url: Url,
    pub l1_network_url: Url,
}

impl Devnet {
    pub fn new(url: Url, l1_network_url: Url) -> Result<Self, DevnetError> {
        Ok(Self {
            url,
            l1_network_url,
        })
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
    fn dump(&self, dump_path: DumpPath) -> impl Future<Output = Result<(), DevnetError>> + Send;
    fn load(&self, load_path: LoadPath) -> impl Future<Output = Result<(), DevnetError>> + Send;
    fn set_time(
        &self,
        set_time_params: SetTimeParams,
    ) -> impl Future<Output = Result<SetTimeResponse, DevnetError>> + Send;
    fn increase_time(
        &self,
        increase_time_params: IncreaseTimeParams,
    ) -> impl Future<Output = Result<IncreaseTimeResponse, DevnetError>> + Send;
    fn mint(
        &self,
        mint_params: MintTokensParams,
    ) -> impl Future<Output = Result<MintTokensResponse, DevnetError>> + Send;
    fn create_block(&self)
        -> impl Future<Output = Result<CreateBlockResponse, DevnetError>> + Send;
    fn abort_blocks(
        &self,
        abort_blocks_params: AbortBlocksParams,
    ) -> impl Future<Output = Result<AbortBlocksResponse, DevnetError>> + Send;
    fn restart(&self) -> impl Future<Output = Result<(), DevnetError>> + Send;
    fn postman_load_l1_messaging_contract(
        &self,
        params: PostmanLoadL1MessagingContractParams,
    ) -> impl Future<Output = Result<PostmanLoadL1MessagingContractResponse, DevnetError>> + Send;
    fn postman_flush(
        &self,
        postman_flush_params: PostmanFlushParameters,
    ) -> impl Future<Output = Result<PostmanFlushResponse, DevnetError>> + Send;
    fn postman_send_message_to_l2(
        &self,
        postman_send_message_to_l2_params: MsgToL2<Felt>,
    ) -> impl Future<Output = Result<PostmanSendMessageToL2Response, DevnetError>> + Send;
    fn devnet_config(
        &self,
    ) -> impl Future<Output = Result<DevnetConfigResponse, DevnetError>> + Send;
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
                )?,
                unit: Some(PriceUnit::Wei),
            },
        )
        .await
    }

    async fn predeployed_accounts(&self) -> Result<Vec<SerializableAccount>, DevnetError> {
        endpoints::get_predeployed_accounts(self.url.clone()).await
    }
    async fn dump(&self, dump_path: DumpPath) -> Result<(), DevnetError> {
        endpoints::dump(self.url.clone(), dump_path).await
    }
    async fn load(&self, load_path: LoadPath) -> Result<(), DevnetError> {
        endpoints::load(self.url.clone(), load_path).await
    }
    async fn postman_load_l1_messaging_contract(
        &self,
        params: PostmanLoadL1MessagingContractParams,
    ) -> Result<PostmanLoadL1MessagingContractResponse, DevnetError> {
        endpoints::postman_load_l1_messaging_contract(self.url.clone(), params).await
    }
    async fn set_time(
        &self,
        set_time_params: SetTimeParams,
    ) -> Result<SetTimeResponse, DevnetError> {
        endpoints::set_time(self.url.clone(), set_time_params).await
    }
    async fn increase_time(
        &self,
        increase_time_params: IncreaseTimeParams,
    ) -> Result<IncreaseTimeResponse, DevnetError> {
        endpoints::increase_time(self.url.clone(), increase_time_params).await
    }
    async fn mint(&self, mint_params: MintTokensParams) -> Result<MintTokensResponse, DevnetError> {
        endpoints::mint(self.url.clone(), mint_params).await
    }
    async fn create_block(&self) -> Result<CreateBlockResponse, DevnetError> {
        endpoints::create_block(self.url.clone()).await
    }
    async fn abort_blocks(
        &self,
        abort_blocks_params: AbortBlocksParams,
    ) -> Result<AbortBlocksResponse, DevnetError> {
        endpoints::abort_blocks(self.url.clone(), abort_blocks_params).await
    }
    async fn restart(&self) -> Result<(), DevnetError> {
        endpoints::restart(self.url.clone()).await
    }

    async fn postman_flush(
        &self,
        postman_flush_params: PostmanFlushParameters,
    ) -> Result<PostmanFlushResponse, DevnetError> {
        endpoints::postman_flush(self.url.clone(), postman_flush_params).await
    }
    async fn postman_send_message_to_l2(
        &self,
        postman_send_message_to_l2_params: MsgToL2<Felt>,
    ) -> Result<PostmanSendMessageToL2Response, DevnetError> {
        endpoints::postman_send_message_to_l2(self.url.clone(), postman_send_message_to_l2_params)
            .await
    }
    async fn devnet_config(&self) -> Result<DevnetConfigResponse, DevnetError> {
        endpoints::devnet_config(self.url.clone()).await
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn test_devnet_endpoints(
    url: Url,
    l1_network_url: Url,
    sierra_path: &str,
    casm_path: &str,
    class_hash: Option<Felt>,
    account_address: Option<Felt>,
    private_key: Option<Felt>,
    erc20_strk_contract_address: Option<Felt>,
    erc20_eth_contract_address: Option<Felt>,
    amount_per_test: Option<Felt>,
) -> Result<(), DevnetError> {
    info!("{}", "⌛ Testing Devnet V7 endpoints -- START ⌛".yellow());

    let devnet = Devnet::new(url.clone(), l1_network_url)?;

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
            path: "starknet-rpc-tests/checker/assets/load_path.json".to_string(),
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
            )?,
            amount: 244837814099629,
            unit: Some(PriceUnit::Wei),
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

    match devnet.create_block().await {
        Ok(create_block_response) => {
            let block_hash = create_block_response.block_hash;

            match devnet
                .abort_blocks(AbortBlocksParams {
                    starting_block_hash: block_hash,
                })
                .await
            {
                Ok(_) => {
                    info!(
                        "{} {}",
                        "✓ Devnet abort_block COMPATIBLE".green(),
                        "✓".green()
                    );
                }
                Err(e) => error!(
                    "{} {} {}",
                    "✗ Devnet abort_block INCOMPATIBLE:".red(),
                    e.to_string().red(),
                    "✗".red()
                ),
            }
        }
        Err(e) => {
            error!(
                "{} {} {}",
                "✗ Devnet abort_block INCOMPATIBLE - can't create block:".red(),
                e.to_string().red(),
                "✗".red()
            );
        }
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

    match devnet
        .postman_load_l1_messaging_contract(PostmanLoadL1MessagingContractParams {
            network_url: devnet.l1_network_url.clone().to_string(),
            address: Some("0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".to_string()),
        })
        .await
    {
        Ok(_) => {
            info!(
                "{} {}",
                "✓ Devnet postman_load_l1_messaging_contract COMPATIBLE".green(),
                "✓".green()
            );
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
            );
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Devnet postman_flush INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    match prepare_postman_send_message_to_l2(
        url.clone(),
        sierra_path,
        casm_path,
        devnet.l1_network_url.clone(),
        class_hash,
        account_address,
        private_key,
        erc20_strk_contract_address,
        erc20_eth_contract_address,
        amount_per_test,
    )
    .await
    {
        Ok(msg_to_l2) => match devnet.postman_send_message_to_l2(msg_to_l2).await {
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
        },
        Err(e) => {
            error!(
                "{} {} {}",
                "✗ Devnet postman_send_message_to_l2 INCOMPATIBLE - can't prepare message:".red(),
                e.to_string().red(),
                "✗".red()
            );
        }
    }

    match devnet.devnet_config().await {
        Ok(_) => {
            info!(
                "{} {}",
                "✓ Devnet devnet_config COMPATIBLE".green(),
                "✓".green()
            );
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Devnet devnet_config INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    restart_devnet(url).await?;

    info!("{}", "🏁 Testing Devnet V7 endpoints -- END 🏁".yellow());
    Ok(())
}
