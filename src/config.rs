use crate::job::Job;
use log::error;
use serde::Deserialize;
use std::{fs, process};

const DEFAULT_CONFIG_PATH: &str = "config.yaml";

#[derive(Deserialize)]
struct Config {
    macaroon_path: String,
    cert_path: String,
    server_url: String,
    jobs: Option<Vec<ConfigJob>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ConfigJob {
    pub name: Option<String>,
    pub cron_expression: String,
    pub amount_sats: u32,
    pub ln_address_or_lnurl: String,
    pub max_fee_sats: Option<i64>,
    pub memo: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ValidConfig {
    pub macaroon_path: String,
    pub cert_path: String,
    pub server_url: String,
    pub jobs: Vec<Job>,
}

impl ValidConfig {
    pub async fn new(file_path: Option<&str>) -> Self {
        let path = file_path.unwrap_or(DEFAULT_CONFIG_PATH);

        let config_raw_result = fs::read_to_string(path);

        let config_data = match config_raw_result {
            Ok(config_raw) => config_raw,
            Err(err) => {
                error!("Failed to read config due to: {:?}", err);

                process::exit(1);
            }
        };

        let config_result = serde_yaml::from_str::<Config>(&config_data);

        let config = match config_result {
            Err(error) => {
                error!("Failed to parse config due to: {:?}", error);

                process::exit(1);
            }
            Ok(config) => config,
        };

        let jobs = match &config.jobs {
            None => {
                error!("No jobs to run. Add a job in {}", DEFAULT_CONFIG_PATH);

                process::exit(1);
            }
            Some(jobs) => jobs.iter().map(|job| Job::new(job.to_owned())).collect(),
        };

        tonic_lnd::connect(
            config.server_url.to_owned(),
            &config.cert_path,
            &config.macaroon_path,
        )
        .await
        .expect("Failed to verify connection to LND node.");

        Self {
            cert_path: config.cert_path,
            macaroon_path: config.macaroon_path,
            server_url: config.server_url,
            jobs,
        }
    }
}
