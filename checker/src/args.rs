use clap::{Parser, ValueEnum};

use url::Url;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None, disable_version_flag = true)]
pub struct Args {
    #[arg(long, short, env)]
    pub url: Url,

    #[arg(long, short, env)]
    pub l1_network_url: Url,

    #[arg(long, short, env)]
    pub sierra_path: String,

    #[arg(long, short, env)]
    pub casm_path: String,

    #[arg(long, short, env)]
    pub version: Version,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum Version {
    V5,
    V6,
    V7,
}
