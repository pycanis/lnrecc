use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use cron::Schedule;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, process, str::FromStr, time::Duration};

const DEFAULT_CONFIG: &str = "config.yaml";

fn read_config(file_path: Option<&str>) -> Config {
    let path = file_path.unwrap_or(DEFAULT_CONFIG);

    if !Path::new(path).exists() {
        let default_config = r#"macaroon_path: "/path/to/macaroon"
cert_path: "/path/to/cert"
jobs:
#  - name: "My first job"
#    schedule: "0 30 9,12,15 1,15 May-Aug Mon,Wed,Fri 2018/2"
#    amount_in_sats: 10000"#;

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

    for job in config.jobs.as_deref().unwrap_or(&[]) {
        let schedule = Schedule::from_str(&job.cron_expression);

        if schedule.is_err() {
            println!("Job has an invalid schedule: {}", &job.cron_expression);

            process::exit(1);
        }
    }

    config
}

async fn run_scheduler(config: &Config) {
    match &config.jobs {
        None => {
            println!("No jobs to run. Add a job in {}", DEFAULT_CONFIG);
            // todo: somehow keep the program hanging, waiting for config update
            return;
        }
        Some(config_jobs) => {
            let mut jobs: Vec<Job> = config_jobs
                .iter()
                .map(|config_job| Job::new(config_job.to_owned()))
                .collect();

            loop {
                // todo: consider finding all the jobs that are due to run
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

                        tokio::spawn(async move {
                            job_clone.run().await;
                        });
                    }
                    None => break,
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    macaroon_path: String,
    cert_path: String,
    jobs: Option<Vec<ConfigJob>>,
}

#[derive(Debug, Clone)]
struct Job {
    config_job: ConfigJob,
    schedule: Schedule,
    next_run: Option<DateTime<Utc>>,
    last_run: Option<DateTime<Utc>>,
}

impl Job {
    fn new(config_job: ConfigJob) -> Self {
        let schedule = Schedule::from_str(&config_job.cron_expression).unwrap(); // this should work, already validated in read_config

        let next_run = schedule.upcoming(Utc).next();

        Self {
            config_job,
            schedule,
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

    async fn run(&self) {
        println!(
            "Running job: {:?} at {:?}",
            self.config_job.name,
            Utc::now()
        );

        tokio::time::sleep(Duration::from_secs(2)).await;

        println!("Finished job: {:?}", self.config_job.name);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ConfigJob {
    name: Option<String>,
    cron_expression: String,
    amount_in_sats: u32,
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

    let config = read_config(cli.config_path.as_deref());

    println!("{:?}", config);

    run_scheduler(&config).await;
}
