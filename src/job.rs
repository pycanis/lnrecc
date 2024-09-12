use chrono::{DateTime, Utc};
use cron::Schedule;
use std::{process, str::FromStr};

use crate::{
    config::{ConfigJob, ValidConfig},
    lnurl::{get_url_from_ln_address_or_lnurl, LnurlService},
    nodes::lnd::pay_invoice,
};

#[derive(Debug, Clone)]
pub struct Job {
    config_job: ConfigJob,
    schedule: Schedule,
    url: String,
    pub next_run: Option<DateTime<Utc>>,
    pub last_run: Option<DateTime<Utc>>,
}

impl Job {
    pub fn new(config_job: ConfigJob) -> Self {
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

    pub fn schedule_next(&mut self) {
        self.last_run = self.next_run;

        self.next_run = self
            .schedule
            .upcoming(Utc)
            .find(|time| time != &self.last_run.unwrap());
    }

    pub async fn run(&self, config: ValidConfig) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "Running job: {:?} at {:?}",
            self.config_job.name,
            Utc::now()
        );

        let invoice = LnurlService::new(self.config_job.clone())
            .get_invoice(&self.url)
            .await?;

        pay_invoice(invoice, &self.config_job, config).await?;

        println!("Finished job: {:?}", self.config_job.name);

        Ok(())
    }
}
