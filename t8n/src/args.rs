use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(long, short, env)]
    pub acc_path: String,

    #[arg(long, short, env)]
    pub txns_path: String,

    #[arg(long, short, env, default_value = "./target/t8n/output.json")]
    pub state_path: String,
}
