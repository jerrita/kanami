// https://docs.sayu-bot.com/CodeAdapter/Protocol.html

use crate::{
    config,
    protocol::event::{Event, MessageEvent},
};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;

mod daemon;
mod model;
use model::*;

pub struct GSCoreAdapter {
    sender: Option<mpsc::Sender<MessageReceive>>,
    connection_starting: bool,
}

#[async_trait]
impl super::Application for GSCoreAdapter {
    fn name(&self) -> &str {
        "gscore adapter"
    }

    async fn on_load(&mut self) -> Result<()> {
        log::info!("app <{}> loaded", self.name());
        self.start_gscore_connection().await?;
        Ok(())
    }

    async fn on_event(&mut self, event: Arc<Event>) -> Result<()> {
        if let Event::MessageEvent(msg_event) = event.as_ref() {
            if let MessageEvent::Group(e) = msg_event {
                if !config::GSCORE_ENABLED_GROUPS.contains(&e.group_id) {
                    return Ok(());
                }
            }
            if let Some(sender) = &self.sender {
                let message_receive = msg_event.into();
                if let Err(e) = sender.send(message_receive).await {
                    log::info!("GSCore handler unavailable ({}), restarting connection...", e);
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
    pub fn new() -> Self {
        Self { 
            sender: None,
            connection_starting: false,
        }
    }

    async fn start_gscore_connection(&mut self) -> Result<()> {
        if self.sender.is_some() || self.connection_starting {
            return Ok(()); // Already connected or connecting
        }

        self.connection_starting = true;
        let (tx, rx) = mpsc::channel(100); // Increase buffer size
        self.sender = Some(tx);
        self.connection_starting = false;

        tokio::spawn(async move {
            daemon::gscore_loop(rx).await;
        });

        Ok(())
    }
}
