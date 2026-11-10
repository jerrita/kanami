use anyhow::Result;
use async_trait::async_trait;
use std::{sync::Arc, time::SystemTime};

use crate::{
    config,
    protocol::{adapter::ROUND_START_TIME, event::Event},
};

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {}

pub struct PingApp;

#[async_trait]
impl super::Application for PingApp {
    fn name(&self) -> &str {
        "ping"
    }

    async fn on_event(&mut self, event: Arc<Event>) -> Result<()> {
        if let Event::MessageEvent(event) = event.as_ref() {
            if event.user_id() == config::get().owner && event.raw_message() == "ping" {
                event.reply("pong", true).await?;
            }
            if event.raw_message() == "!perf" {
                let cur_time = SystemTime::now();
                let round_time = *ROUND_START_TIME.lock().await;
                let dur = cur_time.duration_since(round_time)?;
                event.reply(format!("tpr: {:?}", dur), true).await?;
            }
        }
        Ok(())
    }
}

impl PingApp {
    pub fn new() -> Self {
        Self {}
    }
}
