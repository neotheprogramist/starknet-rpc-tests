use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Args {
    #[arg(short, long, env)]
    pub file_path: PathBuf,

    #[arg(short, long, env)]
    pub public_key: Option<String>,

    #[arg(short, long, env)]
    pub chain_id: String,
}
