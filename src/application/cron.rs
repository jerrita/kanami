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

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub jobs: Vec<JobConfig>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobConfig {
    pub schedule: String,
    pub content: String,
    pub target: Target,
}

#[derive(Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Target {
    MainGroup {},
    Group { id: i64 },
    Private { id: i64 },
}

pub struct CronApp {
    config: Config,
    sched: Option<JobScheduler>,
}

#[async_trait]
impl Application for CronApp {
    const NAME: &'static str = "cron";
    type Config = Config;

    fn new(config: Config) -> Result<Self> {
        Ok(Self {
            config,
            sched: None,
        })
    }

    async fn start(&mut self) -> Result<()> {
        let sched = JobScheduler::new().await?;

        for job in &self.config.jobs {
            let scheduled_job = job.clone();
            sched
                .add(Job::new_async(job.schedule.as_str(), move |_uuid, _l| {
                    let job = scheduled_job.clone();
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
        Ok(())
    }

    async fn on_event(&mut self, _event: Arc<Event>) -> Result<()> {
        Ok(())
    }
}
