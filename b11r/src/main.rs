pub mod args;
pub mod block_build;
pub mod utils;
use args::Args;
use block_build::{block::build_block_tx_hashes_thin, errors::Error};
use clap::Parser;
use utils::{read_input_file, write_block_file};

fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args = Args::parse();

    let b11r_input = read_input_file(args.input_path)?;

    let block = build_block_tx_hashes_thin(b11r_input)?;

    write_block_file(args.output_path, &block)?;

    Ok(())
}
