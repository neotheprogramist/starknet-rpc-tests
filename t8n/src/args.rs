use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(long, short, env, required_unless_present = "forwarded_state")]
    pub acc_path: Option<PathBuf>, // Optional when forwarded_state is true

    #[arg(long, short, env)]
    pub txns_path: PathBuf,

    #[arg(long, short, env, default_value = "./target/t8n/output.json")]
    pub state_path: PathBuf,

    /// This parameter allows the program to accept input state from the output of a previous t8n run (which is state).
    #[arg(long, short)]
    pub forwarded_state: bool,
}
