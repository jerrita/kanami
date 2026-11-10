use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use crate::protocol::event::Event;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {}

struct TemplateApp;

#[async_trait]
impl super::Application for TemplateApp {
    const NAME: &'static str = "template";
    type Config = Config;

    fn new(_config: Config) -> Result<Self> {
        Ok(Self)
    }

    async fn on_event(&mut self, event: Arc<Event>) -> Result<()> {
        todo!()
    }
}
