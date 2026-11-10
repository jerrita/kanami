use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{
    application::Application,
    config,
    protocol::{event::Event, get_bot},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub jobs: Vec<JobConfig>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobConfig {
    pub schedule: String,
    pub content: String,
    pub target: Target,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Target {
    MainGroup {},
    Group { id: i64 },
    Private { id: i64 },
}

pub struct CronApp {
    config: &'static Config,
    sched: Option<JobScheduler>,
}

#[async_trait]
impl Application for CronApp {
    fn name(&self) -> &str {
        "cron"
    }

    async fn on_load(&mut self) -> Result<()> {
        let sched = JobScheduler::new().await?;

        for job in &self.config.jobs {
            sched
                .add(Job::new_async(job.schedule.as_str(), move |_uuid, _l| {
                    Box::pin(async move {
                        let bot = get_bot().await;
                        let result = match job.target {
                            Target::MainGroup {} => {
                                bot.send_group_message(
                                    config::get().main_group,
                                    job.content.as_str(),
                                )
                                .await
                            }
                            Target::Group { id } => {
                                bot.send_group_message(id, job.content.as_str()).await
                            }
                            Target::Private { id } => {
                                bot.send_private_message(id, job.content.as_str()).await
                            }
                        };
                        if let Err(e) = result {
                            log::error!("failed to send prompt: {}", e);
                        }
                    })
                })?)
                .await?;
        }

        sched.start().await?;
        self.sched = Some(sched);
        log::info!("app <{}> loaded", self.name());
        Ok(())
    }

    async fn on_event(&mut self, _event: Arc<Event>) -> Result<()> {
        Ok(())
    }
}

impl CronApp {
    pub fn new(config: &'static Config) -> Self {
        Self {
            config,
            sched: None,
        }
    }
}
