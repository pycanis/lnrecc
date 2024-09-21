use crate::job::Job;
use log::error;
use serde::Deserialize;
use std::{env, fs, path::Path, process};

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

        if !Path::new(path).exists() {
            let default_config = r#"macaroon_path: "~/.lnd/data/chain/bitcoin/mainnet/admin.macaroon"
cert_path: "~/.lnd/tls.cert"
server_url: "https://localhost:10009"
jobs:
#  - name: "My first job"
#    schedule: "0 30 9,12,15 1,15 May-Aug Mon,Wed,Fri 2018/2"
#    amount_sats: 10000
#    ln_address_or_lnurl: "nick@domain.com"
#    max_fee_sats: 5
#    memo: "Scheduled payment coming your way!""#;

            fs::write(path, default_config).expect("Failed to create default config");
        }

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

        let home_dir = env::var("HOME").unwrap_or("~".to_string());

        let cert_path = config.cert_path.replacen("~", &home_dir, 1);
        let macaroon_path = config.macaroon_path.replacen("~", &home_dir, 1);

        tonic_lnd::connect(config.server_url.to_owned(), &cert_path, &macaroon_path)
            .await
            .expect("Failed to verify connection to LND node.");

        Self {
            cert_path,
            macaroon_path,
            server_url: config.server_url,
            jobs,
        }
    }
}
