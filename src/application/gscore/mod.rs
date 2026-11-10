// https://docs.sayu-bot.com/CodeAdapter/Protocol.html

use crate::{config, protocol::event::Event};
use anyhow::Result;
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
    connection_starting: bool,
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
            connection_starting: false,
        })
    }

    async fn start(&mut self) -> Result<()> {
        self.start_gscore_connection().await?;
        Ok(())
    }

    async fn on_event(&mut self, event: Arc<Event>) -> Result<()> {
        if let Event::MessageEvent(msg_event) = event.as_ref() {
            if let Some(sender) = &self.sender {
                let message_receive = MessageReceive::new(msg_event, &self.config);
                if let Err(e) = sender.send(message_receive).await {
                    log::info!(
                        "GSCore handler unavailable ({}), restarting connection...",
                        e
                    );
                    // 重新启动连接
                    self.sender = None;
                    if let Err(restart_err) = self.start_gscore_connection().await {
                        log::warn!("Failed to restart GSCore connection: {}", restart_err);
                    }
                }
            }
        }
        Ok(())
    }
}

impl GSCoreAdapter {
    async fn start_gscore_connection(&mut self) -> Result<()> {
        if self.sender.is_some() || self.connection_starting {
            return Ok(()); // Already connected or connecting
        }

        self.connection_starting = true;
        let (tx, rx) = mpsc::channel(100); // Increase buffer size
        self.sender = Some(tx);
        self.connection_starting = false;

        let config = self.config.clone();
        tokio::spawn(async move {
            daemon::gscore_loop(rx, config).await;
        });

        Ok(())
    }
}
