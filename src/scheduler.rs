pub mod scheduler {
    use std::time::Duration;

    use chrono::Utc;

    use crate::{config::config::ValidConfig, job::job::Job};

    pub async fn run_scheduler(config: &ValidConfig) {
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
                        let _ = job_clone.run(config_clone).await;
                    });
                }
                None => {
                    println!("No more jobs to execute. Exiting..");

                    break;
                }
            }
        }
    }
}