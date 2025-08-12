use crate::protocol::{
    event::{self, Event, MessageEvent},
    get_bot,
};
use anyhow::Result;
use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};

pub struct BuiltinApp {
    group_map: HashMap<i64, String>,
}

#[async_trait]
impl super::Application for BuiltinApp {
    fn name(&self) -> &str {
        "builtin"
    }

    async fn on_event(&mut self, event: Arc<event::Event>) -> Result<()> {
        match event.as_ref() {
            Event::MessageEvent(event) => match event {
                MessageEvent::Group(event) => {
                    let group_name = self.get_group_name(event.group_id).await?;
                    log::info!(
                        "{}({}): {}({}) -> {}",
                        group_name,
                        event.group_id,
                        event.sender.nickname,
                        event.sender.user_id,
                        event.message
                    )
                }

                MessageEvent::Private(event) => {
                    log::info!(
                        "{}({}) -> {}",
                        event.sender.nickname,
                        event.sender.user_id,
                        event.message
                    );
                }
            },
            _ => {}
        };
        Ok(())
    }
}

impl BuiltinApp {
    pub fn new() -> Self {
        Self {
            group_map: HashMap::new(),
        }
    }

    async fn get_group_name(&mut self, group_id: i64) -> Result<&String> {
        if !self.group_map.contains_key(&group_id) {
            let bot = get_bot().await;
            let res = bot.get_group_info(group_id, false).await?;
            let group_name = res
                .data
                .as_ref()
                .and_then(|d| d.get("group_name"))
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown>")
                .to_string();
            self.group_map.insert(group_id, group_name);
        }

        Ok(unsafe { self.group_map.get(&group_id).unwrap_unchecked() })
    }
}
