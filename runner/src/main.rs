mod args;
mod errors;
mod transports;
mod utils;
use args::Args;
use clap::Parser;
mod tests;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _args: Args = Args::parse();

    Ok(())
}
