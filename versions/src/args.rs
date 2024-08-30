use clap::Parser;

use url::Url;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(long, short, env)]
    pub url: Url,

    #[arg(long, short, env)]
    pub sierra_path: String,

    #[arg(long, short, env)]
    pub casm_path: String,
}
