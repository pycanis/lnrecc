use clap::Parser;
use cli::Cli;
use config::ValidConfig;
use scheduler::run_scheduler;

mod cli;
mod config;
mod job;
mod lnurl;
mod nodes;
mod scheduler;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    println!("{:?}", cli);

    let config = ValidConfig::new(cli.config_path.as_deref()).await;

    println!("{:?}", config);

    run_scheduler(&config).await;
}
