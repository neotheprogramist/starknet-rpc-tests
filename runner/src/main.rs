mod args;

use args::Args;
use clap::Parser;

use shared::account_balance::Version;

use v0_0_5::endpoints::{
    account_balance::account_balance as account_balance_v5,
    add_declare_transaction::add_declare_transaction as add_declare_transaction_v5,
    block_number::block_number as block_number_v5, chain_id::get_chain_id as get_chain_id_v5,
    get_block_with_tx_hashes::get_block_with_tx_hashes as get_block_with_tx_hashes_v5,
    get_block_with_txs::get_block_with_txs as get_block_with_txs_v5,
    get_class_cairo_1::get_class_cairo_1 as get_class_cairo_1_v5,
    get_nonce::get_nonce as get_nonce_v5,
    get_state_update::get_state_update as get_state_update_v5,
    get_storage_at::get_storage_at as get_storage_at_v5,
    get_transaction_by_hash_declare_v3::get_transaction_by_hash_declare_v3 as get_transaction_by_hash_declare_v3_v5,
    specversion::specversion as specversion_v5,
};

use v0_0_6::endpoints::{
    account_balance::account_balance as account_balance_v6,
    add_declare_transaction::add_declare_transaction as add_declare_transaction_v6,
    block_number::block_number as block_number_v6, chain_id::get_chain_id as get_chain_id_v6,
    get_block_with_tx_hashes::get_block_with_tx_hashes as get_block_with_tx_hashes_v6,
    get_block_with_txs::get_block_with_txs as get_block_with_txs_v6,
    get_class_cairo_1::get_class_cairo_1 as get_class_cairo_1_v6,
    get_nonce::get_nonce as get_nonce_v6,
    get_state_update::get_state_update as get_state_update_v6,
    get_storage_at::get_storage_at as get_storage_at_v6,
    get_transaction_by_hash_declare_v3::get_transaction_by_hash_declare_v3 as get_transaction_by_hash_declare_v3_v6,
    specversion::specversion as specversion_v6,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    use v0_0_5::endpoints::{
        account_balance::account_balance as account_balance_v5,
        add_declare_transaction::add_declare_transaction as add_declare_transaction_v5,
        block_number::block_number as block_number_v5, chain_id::get_chain_id as get_chain_id_v5,
        get_block_with_tx_hashes::get_block_with_tx_hashes as get_block_with_tx_hashes_v5,
        get_block_with_txs::get_block_with_txs as get_block_with_txs_v5,
        get_class_cairo_1::get_class_cairo_1 as get_class_cairo_1_v5,
        get_nonce::get_nonce as get_nonce_v5,
        get_state_update::get_state_update as get_state_update_v5,
        get_storage_at::get_storage_at as get_storage_at_v5,
        get_transaction_by_hash_declare_v3::get_transaction_by_hash_declare_v3 as get_transaction_by_hash_declare_v3_v5,
        specversion::specversion as specversion_v5,
    };

    use v0_0_6::endpoints::{
        account_balance::account_balance as account_balance_v6,
        add_declare_transaction::add_declare_transaction as add_declare_transaction_v6,
        block_number::block_number as block_number_v6, chain_id::get_chain_id as get_chain_id_v6,
        get_block_with_tx_hashes::get_block_with_tx_hashes as get_block_with_tx_hashes_v6,
        get_block_with_txs::get_block_with_txs as get_block_with_txs_v6,
        get_class_cairo_1::get_class_cairo_1 as get_class_cairo_1_v6,
        get_nonce::get_nonce as get_nonce_v6,
        get_state_update::get_state_update as get_state_update_v6,
        get_storage_at::get_storage_at as get_storage_at_v6,
        get_transaction_by_hash_declare_v3::get_transaction_by_hash_declare_v3 as get_transaction_by_hash_declare_v3_v6,
        specversion::specversion as specversion_v6,
    };
    let args = Args::parse();
    let (url, chain_id, vers) = (args.url.clone(), args.chain_id.clone(), args.vers);
    match vers {
        Version::V0_0_5 => {
            match specversion_v5(url.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            };

            match get_nonce_v5(url.clone(), chain_id.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            };

            match get_block_with_tx_hashes_v5(url.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            };

            match block_number_v5(url.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            };

            match get_block_with_txs_v5(url.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            }

            match get_state_update_v5(url.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            }

            match get_storage_at_v5(url.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            }

            match get_transaction_by_hash_declare_v3_v5(url.clone(), chain_id.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            }

            match get_class_cairo_1_v5(url.clone(), chain_id.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            }

            match get_chain_id_v5(url.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            }

            match add_declare_transaction_v5(url.clone(), chain_id.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            };

            match account_balance_v5(url.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            };
        }
        Version::V0_0_6 => {
            match specversion_v6(url.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            };

            match get_nonce_v6(url.clone(), chain_id.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            };

            match get_block_with_tx_hashes_v6(url.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            };

            match block_number_v6(url.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            };

            match get_block_with_txs_v6(url.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            }

            match get_state_update_v6(url.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            }

            match get_storage_at_v6(url.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            }

            match get_transaction_by_hash_declare_v3_v6(url.clone(), chain_id.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            }

            match get_class_cairo_1_v6(url.clone(), chain_id.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            }

            match get_chain_id_v6(url.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            }

            match add_declare_transaction_v6(url.clone(), chain_id.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            };

            match account_balance_v6(url.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(());
                }
            };
        }
    }

    Ok(())
}
