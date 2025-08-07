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
    sender: Option<mpsc::UnboundedSender<MessageReceive>>,
}

#[async_trait]
impl super::Application for GSCoreAdapter {
    fn name(&self) -> &str {
        "gscore adapter"
    }

    async fn on_load(&mut self) -> Result<()> {
        log::info!("app <{}> loaded", self.name());

        // 启动GSCore连接任务
        let (tx, rx) = mpsc::unbounded_channel();
        self.sender = Some(tx);

        tokio::spawn(async move {
            daemon::gscore_loop(rx).await;
        });

        Ok(())
    }

    async fn on_event(&mut self, event: Arc<Event>) -> Result<()> {
        if let Event::MessageEvent(msg_event) = event.as_ref() {
            if let MessageEvent::Group(e) = msg_event {
                if e.group_id != config::GSCORE_ENABLED_GROUP {
                    return Ok(());
                }
            }
            if let Some(sender) = &self.sender {
                let message_receive = msg_event.into();
                if let Err(e) = sender.send(message_receive) {
                    log::warn!("Failed to send message to GSCore handler: {}", e);
                }
            }
        }
        Ok(())
    }
}

impl GSCoreAdapter {
    pub fn new() -> Self {
        Self { sender: None }
    }
}
