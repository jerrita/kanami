use anyhow::Result;
use async_trait::async_trait;
use std::{sync::Arc, time::SystemTime};

use crate::{
    config,
    protocol::{
        adapter::ROUND_START_TIME,
        event::{Event, MessageEvent},
    },
};

pub struct PingApp;

#[async_trait]
impl super::Application for PingApp {
    fn name(&self) -> &str {
        "ping"
    }

    async fn on_load(&mut self) -> Result<()> {
        log::info!("app <{}> loaded", self.name());
        Ok(())
    }

    async fn on_event(&mut self, event: Arc<Event>) -> Result<()> {
        match event.as_ref() {
            Event::MessageEvent(MessageEvent::Private(event)) => {
                if event.sender.user_id == config::OWNER && event.raw_message == "ping" {
                    event.reply("pong", true).await?;
                }
                if event.raw_message == "!perf" {
                    let cur_time = SystemTime::now();
                    let round_time = *ROUND_START_TIME.lock().await;
                    let dur = cur_time.duration_since(round_time)?;
                    event.reply(format!("tpr: {:?}", dur), true).await?;
                }
            }
            Event::MessageEvent(MessageEvent::Group(event)) => {
                if event.group_id == config::MAIN_GROOUP && event.raw_message == "ping" {
                    event.reply("pong", true).await?;
                }
                if event.raw_message == "!perf" {
                    let cur_time = SystemTime::now();
                    let round_time = *ROUND_START_TIME.lock().await;
                    let dur = cur_time.duration_since(round_time)?;
                    event.reply(format!("tpr: {:?}", dur), true).await?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl PingApp {
    pub fn new() -> Self {
        Self {}
    }
}
