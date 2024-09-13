use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "b11r")]
pub struct Args {
    #[arg(short, long, env)]
    pub block_header_path: PathBuf,

    #[arg(short, long, env)]
    pub transactions_path: PathBuf,

    #[arg(short, long, env)]
    pub receipt_path: PathBuf,

    #[arg(short, long, env)]
    pub state_diff_path: PathBuf,


}