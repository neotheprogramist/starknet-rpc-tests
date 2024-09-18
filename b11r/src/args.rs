use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "b11r")]
pub struct Args {
    #[arg(short, long, env)]
    pub input_path: PathBuf,

    #[arg(short, long, env, default_value = "./target/b11r/block.json")]
    pub output_path: PathBuf,
}
