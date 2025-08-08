use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use crate::{
    config,
    protocol::event::{Event, MessageEvent},
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
            }
            Event::MessageEvent(MessageEvent::Group(event)) => {
                if event.group_id == config::MAIN_GROOUP && event.raw_message == "ping" {
                    event.reply("pong", true).await?;
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
