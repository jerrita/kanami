use crate::protocol::{
    event::{self, Event, MessageEvent},
    get_bot,
};
use anyhow::{Result, anyhow};
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

    async fn on_load(&mut self) -> Result<()> {
        log::info!("plugn <{}> loaded", self.name());
        Ok(())
    }

    async fn on_event(&mut self, event: Arc<event::Event>) -> Result<()> {
        match event.as_ref() {
            Event::MessageEvent(event) => match event {
                MessageEvent::Group(event) => {
                    let mut group_name = self.group_map.get(&event.group_id);
                    if group_name.is_none() {
                        let bot = get_bot().await;
                        let res = bot.get_group_info(event.group_id, false).await?;
                        self.group_map.insert(
                            event.group_id,
                            res.data
                                .get("group_name")
                                .ok_or(anyhow!("failed get group info."))?
                                .as_str()
                                .unwrap_or("<unknown>")
                                .to_string(),
                        );
                        group_name = self.group_map.get(&event.group_id)
                    };
                    log::info!(
                        "{}({}): {}({}) -> {}",
                        unsafe { group_name.unwrap_unchecked() },
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
}
