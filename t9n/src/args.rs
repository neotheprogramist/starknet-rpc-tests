use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[arg(short, long)]
    pub file_path: String,

    #[arg(short, long)]
    pub public_key: String,

    #[arg(short, long)]
    pub chain_id: String,
}
