use starknet_types_rpc::{BlockId, BlockTag, Felt};
use tracing::info;
use url::Url;

use crate::v5::rpc::{
    accounts::{
        account::Account,
        creation::{
            create::{create_account, AccountType},
            helpers::get_chain_id,
            structs::MintRequest,
        },
        deployment::{
            deploy::deploy_account,
            structs::{ValidatedWaitParams, WaitForTx},
        },
        single_owner::{ExecutionEncoding, SingleOwnerAccount},
        utils::mint::mint,
    },
    endpoints::{declare_contract::declare_contract, deploy_contract::deploy_contract},
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
    signers::{key_pair::SigningKey, local_wallet::LocalWallet},
};

pub async fn decalare_and_deploy(
    url: Url,

    chain_id: &str,
    sierra_path: &str,
    casm_path: &str,
) -> Result<(), String> {
    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));
    // let create_acc_data =
    //     match create_account(&provider, AccountType::Oz, Option::None, Option::None).await {
    //         Ok(value) => value,
    //         Err(e) => {
    //             info!("{}", "Could not create an account");
    //             return Err(e.to_string());
    //         }
    //     };
    // info!("{:?}", create_acc_data);
    // match mint(
    //     url.clone(),
    //     &MintRequest {
    //         amount: u128::MAX,
    //         address: create_acc_data.address,
    //     },
    // )
    // .await
    // {
    //     Ok(response) => info!("{} {} {:?}", "Minted tokens", u128::MAX, response),
    //     Err(e) => {
    //         info!("{}", "Could not mint tokens");
    //         return Err(e.to_string());
    //     }
    // };

    // let wait_conifg = WaitForTx {
    //     wait: true,
    //     wait_params: ValidatedWaitParams::default(),
    // };

    // let chain_id = get_chain_id(&provider).await.unwrap();
    // let result =
    //     match deploy_account(&provider, chain_id, wait_conifg, create_acc_data.clone()).await {
    //         Ok(value) => Some(value),
    //         Err(e) => {
    //             info!("{}", "Could not deploy an account");
    //             return Err(e.to_string());
    //         }
    //     };
    // info!("After deploy, resultt: {:?}", result);

    let sender_address =
        Felt::from_hex("0x78662e7352d062084b0010068b99288486c2d8b914f6e2a55ce945f8792c8b1")
            .unwrap();
    let signer: LocalWallet = LocalWallet::from(SigningKey::from_secret_scalar(
        Felt::from_hex("0xe1406455b7d66b1690803be066cbe5e").unwrap(),
    ));
    let chain_id = Felt::from_hex("0x534e5f5345504f4c4941").unwrap();

    let mut account = SingleOwnerAccount::new(
        JsonRpcClient::new(HttpTransport::new(url.clone())),
        signer,
        sender_address,
        chain_id,
        ExecutionEncoding::New,
    );
    account.set_block_id(BlockId::Tag(BlockTag::Pending));

    let class_hash = declare_contract(&account, sierra_path, casm_path)
        .await
        .unwrap();
    info!("declare final result {}", class_hash);
    let deploy_result = deploy_contract(&account, class_hash).await;
    info!("DEPLOY CONTRACT RESULT: {:?}", deploy_result);
    Ok(())
}
// let receipt = client
//     .get_transaction_receipt(deploy_result.transaction_hash)
//     .await
//     .unwrap();
// assert!(receipt.block.is_block());

// let receipt = match receipt.receipt {
//     TransactionReceipt::Deploy(receipt) => receipt,
//     _ => panic!("unexpected receipt response type"),
// };

// match receipt.execution_result {
//     ExecutionResult::Succeeded => {}
//     _ => panic!("unexpected execution result"),
// }
// (account, receipt.contract_address)
