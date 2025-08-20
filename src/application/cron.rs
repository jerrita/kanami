use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{
    application::Application,
    config,
    protocol::{event::Event, get_bot},
};

pub struct CronApp {
    sched: Option<JobScheduler>,
}

async fn send_prompt(content: &str) -> Result<()> {
    let bot = get_bot().await;
    bot.send_group_message(config::MAIN_GROUP, content).await?;
    Ok(())
}

#[async_trait]
impl Application for CronApp {
    fn name(&self) -> &str {
        "cron"
    }

    async fn on_load(&mut self) -> Result<()> {
        let sched = JobScheduler::new().await?;

        sched
            .add(Job::new_async("0 0 0 * * 5", |_uuid, _l| {
                Box::pin(async move {
                    if let Err(e) = send_prompt("周五啦！").await {
                        log::error!("failed to send prompt: {}", e);
                    }
                })
            })?)
            .await?;
        sched
            .add(Job::new_async("0 0 0 * * 1", |_uuid, _l| {
                Box::pin(async move {
                    if let Err(e) = send_prompt("周一啦！").await {
                        log::error!("failed to send prompt: {}", e);
                    }
                })
            })?)
            .await?;

        sched
            .add(Job::new_async("0 0 13 * * *", |_uuid, _l| {
                Box::pin(async move {
                    if let Err(e) = get_bot()
                        .await
                        .send_private_message(config::USER_1, "晚上好！今天记得打卡哦~")
                        .await
                    {
                        log::error!("failed to send prompt: {}", e);
                    }
                })
            })?)
            .await?;

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
    pub fn new() -> Self {
        Self { sched: None }
    }
}
