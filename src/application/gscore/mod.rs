// https://docs.sayu-bot.com/CodeAdapter/Protocol.html

use crate::{config, protocol::event::Event};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::mpsc;

mod daemon;
mod model;
use model::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub endpoint: String,
    pub bot_id: String,
    #[serde(deserialize_with = "config::user_id")]
    pub node_sender_id: i64,
    pub node_sender_nickname: String,
}

pub struct GSCoreAdapter {
    config: Arc<Config>,
    sender: Option<mpsc::Sender<MessageReceive>>,
}

#[async_trait]
impl super::Application for GSCoreAdapter {
    const NAME: &'static str = "gscore";
    type Config = Config;

    fn new(config: Config) -> Result<Self> {
        let config = Arc::new(config);
        Ok(Self {
            config,
            sender: None,
        })
    }

    async fn start(&mut self) -> Result<()> {
        let (sender, receiver) = mpsc::channel(100);
        self.sender = Some(sender);
        let config = self.config.clone();
        tokio::spawn(daemon::gscore_loop(receiver, config));
        Ok(())
    }

    async fn on_event(&mut self, event: Arc<Event>) -> Result<()> {
        if let Event::MessageEvent(msg_event) = event.as_ref() {
            self.sender
                .as_ref()
                .context("gscore has not started")?
                .send(MessageReceive::new(msg_event, &self.config))
                .await?;
        }
        Ok(())
    }
}
