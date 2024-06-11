use crate::v0_0_5::account_balance::AccountBalanceResponseV0_0_5;
use crate::v0_0_6::account_balance::AccountBalanceResponseV0_0_6;
use clap::Parser;
use colored::*;
use reqwest::{Client, Error};
use tracing::info;

#[derive(Parser, Debug, Clone)]
pub enum Version {
    V0_0_6,
    V0_0_5,
}

impl std::str::FromStr for Version {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "v6" => Ok(Version::V0_0_6),
            "v5" => Ok(Version::V0_0_5),
            _ => Err(format!("'{}' is not a valid version", s)),
        }
    }
}

pub struct AccountBalanceParams {
    pub address: String,
    pub unit: String,
    pub block_tag: String,
}

pub async fn account_balance(
    account_balance_params: &AccountBalanceParams,
    version: &Version,
    url: &str,
) -> Result<(), Error> {
    let client = Client::new();
    let url = format!(
        "{}/account_balance?address={}&unit={}&block_tag={}",
        url,
        account_balance_params.address,
        account_balance_params.unit,
        account_balance_params.block_tag
    );
    let res = client.get(&url).send().await?;
    match version {
        Version::V0_0_5 => {
            let account_balance_response = res.json::<AccountBalanceResponseV0_0_5>().await;
            match account_balance_response {
                Ok(_) => info!("{}", "COMPATIBLE".green()),
                Err(_) => info!("{}", "INCOMPATIBLE".red()),
            }
        }
        Version::V0_0_6 => {
            let account_balance_response = res.json::<AccountBalanceResponseV0_0_6>().await;
            match account_balance_response {
                Ok(_) => info!("{}", "COMPATIBLE".green()),
                Err(_) => info!("{}", "INCOMPATIBLE".red()),
            }
        }
    };
    Ok(())
}
