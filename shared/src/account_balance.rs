use crate::errors::RequestOrParseError;
use crate::v0_0_5::account_balance::AccountBalanceResponseV0_0_5;
use crate::v0_0_6::account_balance::AccountBalanceResponseV0_0_6;
use clap::Parser;
use colored::*;
use reqwest::Client;
use tracing::info;
use url::Url;

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
    base_url: Url,
) -> Result<(), RequestOrParseError> {
    let client = Client::new();
    let account_balance_url = match base_url.join("account_balance") {
        Ok(url) => url,
        Err(e) => return Err(e.into()),
    };
    let res = client
        .get(account_balance_url)
        .query(&[
            ("address", &account_balance_params.address),
            ("unit", &account_balance_params.unit),
            ("block_tag", &account_balance_params.block_tag),
        ])
        .send()
        .await?;
    println!("{}", res.url());
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
