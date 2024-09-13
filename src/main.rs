use std::fs::OpenOptions;

use clap::Parser;
use cli::Cli;
use config::ValidConfig;
use log::info;
use scheduler::run_scheduler;

mod cli;
mod config;
mod job;
mod lnurl;
mod nodes;
mod scheduler;

#[tokio::main]
async fn main() {
    let target = Box::new(
        OpenOptions::new()
            .append(true)
            .create(true)
            .open("lnrecc.log")
            .expect("Can't open or create log file"),
    );

    env_logger::Builder::from_default_env()
        .target(env_logger::Target::Pipe(target))
        .filter(None, log::LevelFilter::Info)
        .init();

    info!("Starting lnrecc..");

    let cli = Cli::parse();

    if cli.config_path.is_some() {
        info!("Custom config path: {}", cli.config_path.as_ref().unwrap());
    }

    let config = ValidConfig::new(cli.config_path.as_deref()).await;

    run_scheduler(&config).await;
}
