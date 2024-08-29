pub mod txn_validation;
use txn_validation::validate::validate_txn_json;
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    file_path: String,

    #[arg(short, long)]
    public_key: String,

    #[arg(short, long)]
    chain_id: String,
}

fn main() {
    let args = Args::parse();
    match validate_txn_json(&args.file_path, &args.public_key, &args.chain_id) {
        Ok(json_result) => {
            println!("Validation successful: {}", json_result);
        }
        Err(e) => {
            println!("Validation error: {}", e);
        }
    }
}
