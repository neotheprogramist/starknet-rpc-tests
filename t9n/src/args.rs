use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[arg(short, long, env)]
    pub file_path: String,

    #[arg(short, long, env)]
    pub public_key: String,

    #[arg(short, long, env)]
    pub chain_id: String,
}
