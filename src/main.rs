use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use cron::Schedule;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::Path,
    process,
    str::{from_utf8, FromStr},
    time::Duration,
};
use tonic_lnd::lnrpc::payment::PaymentStatus;

const DEFAULT_CONFIG: &str = "config.yaml";

fn get_url_from_ln_address_or_lnurl(ln_address_or_lnurl: &str) -> String {
    if ln_address_or_lnurl.contains("@") {
        let url_parts: Vec<&str> = ln_address_or_lnurl.split("@").collect();

        format!(
            "https://{}/.well-known/lnurlp/{}",
            url_parts[1], url_parts[0]
        )
    } else {
        let lnurl_upper = ln_address_or_lnurl.to_uppercase();

        let (_hrp, data) = bech32::decode(&lnurl_upper).expect("Failed to decode lnurl");

        let url = from_utf8(&data).expect("Failed to convert lnurl bytes to utf8");

        url.to_string()
    }
}

async fn run_scheduler(config: &ValidConfig) {
    let mut jobs: Vec<Job> = config.jobs.clone();

    loop {
        let next_job = jobs
            .iter_mut()
            // this could be made slightly faster using fold instead of filter and min_by_key
            // but this is more readable and shouldn't be a bottleneck
            .filter(|job| job.next_run.is_some() && job.next_run != job.last_run)
            .min_by_key(|job| job.next_run);

        match next_job {
            Some(job) => {
                let now = Utc::now();

                let job_next_run = job.next_run.unwrap();

                let duration_until_next = job_next_run.signed_duration_since(now);

                let seconds_until_next = duration_until_next.num_seconds() as u64;

                if seconds_until_next > 0 {
                    println!(
                        "Waiting to execute job in {} seconds on {}",
                        seconds_until_next, job_next_run
                    );

                    tokio::time::sleep(Duration::from_secs(seconds_until_next)).await;
                };

                job.schedule_next();

                let job_clone = job.clone();
                let config_clone = config.clone();

                tokio::spawn(async move {
                    job_clone.run(config_clone).await;
                });
            }
            None => break,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LnurlInitResponse {
    callback: String,
    max_sendable: u64,
    min_sendable: u64,
    comment_allowed: u64,
    // ...
}

// #[derive(Deserialize)]
// struct LnurlResponseSuccessAction {
//     tag: String,
//     message: String,
// }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LnurlResponse {
    pr: String,
    //  routes: Vec<String>,
    //  success_action: LnurlResponseSuccessAction,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    macaroon_path: String,
    cert_path: String,
    server_url: String,
    jobs: Option<Vec<ConfigJob>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ConfigJob {
    name: Option<String>,
    cron_expression: String,
    amount_in_sats: u32,
    ln_address_or_lnurl: String,
    memo: Option<String>,
}

#[derive(Debug, Clone)]
struct ValidConfig {
    macaroon_path: String,
    cert_path: String,
    server_url: String,
    jobs: Vec<Job>,
}

impl ValidConfig {
    async fn new(file_path: Option<&str>) -> Self {
        let path = file_path.unwrap_or(DEFAULT_CONFIG);

        if !Path::new(path).exists() {
            let default_config = r#"macaroon_path: "~/.lnd/data/chain/bitcoin/mainnet/admin.macaroon"
cert_path: "~/.lnd/tls.cert"
server_url: "http://127.0.0.1:10009"
jobs:
#  - name: "My first job"
#    schedule: "0 30 9,12,15 1,15 May-Aug Mon,Wed,Fri 2018/2"
#    amount_in_sats: 10000
#    ln_address_or_lnurl: "nick@domain.com"
#    memo: "Scheduled payment coming your way!""#;

            fs::write(path, default_config).expect("Failed to create default config");
        }

        let config_raw_result = fs::read_to_string(path);

        let config_data = match config_raw_result {
            Ok(config_raw) => config_raw,
            Err(err) => {
                println!("Failed to read config due to: {:?}", err);

                process::exit(1);
            }
        };

        let config_result = serde_yaml::from_str::<Config>(&config_data);

        let config = match config_result {
            Err(error) => {
                println!("Failed to parse config due to: {:?}", error);

                process::exit(1);
            }
            Ok(config) => config,
        };

        let jobs = match &config.jobs {
            None => {
                println!("No jobs to run. Add a job in {}", DEFAULT_CONFIG);

                process::exit(1);
            }
            Some(jobs) => jobs.iter().map(|job| Job::new(job.to_owned())).collect(),
        };

        tonic_lnd::connect(
            config.server_url.to_owned(),
            config.cert_path.to_owned(),
            config.macaroon_path.to_owned(),
        )
        .await
        .expect("Failed to verify connection to LND node.");

        Self {
            cert_path: config.cert_path.to_owned(),
            macaroon_path: config.macaroon_path.to_owned(),
            server_url: config.server_url.to_owned(),
            jobs,
        }
    }
}

#[derive(Debug, Clone)]
struct Job {
    config_job: ConfigJob,
    schedule: Schedule,
    url: String,
    next_run: Option<DateTime<Utc>>,
    last_run: Option<DateTime<Utc>>,
}

impl Job {
    fn new(config_job: ConfigJob) -> Self {
        let schedule_result = Schedule::from_str(&config_job.cron_expression);

        let schedule = match schedule_result {
            Ok(schedule) => schedule,
            Err(_err) => {
                println!(
                    "Job has an invalid schedule: {}",
                    &config_job.cron_expression
                );

                process::exit(1);
            }
        };

        let next_run = schedule.upcoming(Utc).next();

        let url = get_url_from_ln_address_or_lnurl(&config_job.ln_address_or_lnurl);

        Self {
            config_job,
            schedule,
            url,
            next_run,
            last_run: None,
        }
    }

    fn schedule_next(&mut self) {
        self.last_run = self.next_run;

        self.next_run = self
            .schedule
            .upcoming(Utc)
            .find(|time| time != &self.last_run.unwrap());
    }

    async fn run(&self, config: ValidConfig) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "Running job: {:?} at {:?}",
            self.config_job.name,
            Utc::now()
        );

        let lnurl_init_response = reqwest::get(self.url.clone())
            .await?
            .json::<LnurlInitResponse>()
            .await?;

        // todo: validate

        let lnurl_request_url = format!(
            "{}?amount={}&comment={}",
            lnurl_init_response.callback,
            self.config_job.amount_in_sats * 1000,
            self.config_job.memo.to_owned().unwrap_or("".to_string())
        );

        let lnurl_response = reqwest::get(lnurl_request_url)
            .await?
            .json::<LnurlResponse>()
            .await?;

        let mut client = tonic_lnd::connect(
            config.server_url.to_owned(),
            config.cert_path.to_owned(),
            config.macaroon_path.to_owned(),
        )
        .await?;

        let payment_response = client
            .router()
            .send_payment_v2(tonic_lnd::routerrpc::SendPaymentRequest {
                payment_request: lnurl_response.pr,
                timeout_seconds: 30,
                fee_limit_sat: (self.config_job.amount_in_sats as f32 * 0.01).ceil() as i64, // max 1% fee
                ..Default::default()
            })
            .await?;

        let mut payment_stream = payment_response.into_inner();

        while let Some(payment) = payment_stream.message().await? {
            println!("Payment update: {:?}", payment);

            let payment_status = PaymentStatus::from_i32(payment.status).unwrap();

            match payment_status {
                PaymentStatus::Succeeded => {
                    println!("Payment success...");
                }
                PaymentStatus::InFlight => {
                    println!("Payment in process...");
                }
                _ => {
                    println!("Payment failed...");
                }
            }
        }

        println!("Finished job: {:?}", self.config_job.name);

        Ok(())
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    config_path: Option<String>,
    // #[command(subcommand)]
    // command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    New {
        #[arg(short, long)]
        name: Option<String>,

        #[arg(short, long)]
        schedule: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    println!("{:?}", cli);

    let config = ValidConfig::new(cli.config_path.as_deref()).await;

    println!("{:?}", config);

    run_scheduler(&config).await;
}
