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
    let cli = Cli::parse();

    let target = match cli.log_path {
        Some(log_path) => {
            let file_target = Box::new(
                OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(log_path)
                    .expect("Can't open or create log file"),
            );

            env_logger::Target::Pipe(file_target)
        }
        None => env_logger::Target::Stdout,
    };

    env_logger::Builder::from_default_env()
        .target(target)
        .filter(None, log::LevelFilter::Info)
        .init();

    info!("Starting lnrecc..");

    if cli.config_path.is_some() {
        info!("Custom config path: {}", cli.config_path.as_ref().unwrap());
    }

    let config = ValidConfig::new(cli.config_path.as_deref()).await;

    run_scheduler(&config).await;
}
