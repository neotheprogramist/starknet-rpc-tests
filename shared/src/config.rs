use crate::account_balance::Version;
use crate::clients::devnet_client::DevnetClient;
use crate::clients::devnet_provider::DevnetProvider;
use crate::errors::RequestOrParseError;
use colored::Colorize;
use tracing::info;
use url::Url;
use utils::transports::http::HttpTransport;

pub async fn get_config(version: &Version) -> Result<(), RequestOrParseError> {
    match version {
        Version::V0_0_5 => {
            let devnet_v5_url = Url::parse("http://localhost:5051").unwrap();
            let client = DevnetClient::new(HttpTransport::new(devnet_v5_url));

            match client.get_config().await {
                Ok(config) => {
                    info!("{}", "COMPATIBLE".green());
                    println!("{:?}", config);
                }
                Err(_) => info!("{}", "INCOMPATIBLE".red()),
            }
        }
        Version::V0_0_6 => {
            let devnet_v6_url: Url = Url::parse("http://localhost:5050").unwrap();
            let client = DevnetClient::new(HttpTransport::new(devnet_v6_url));

            match client.get_config().await {
                Ok(config) => {
                    info!("{}", "COMPATIBLE".green());
                    println!("{:?}", config);
                }
                Err(e) => info!("{} {}", "INCOMPATIBLE".red(), e),
            }
        }
    };
    Ok(())
}
