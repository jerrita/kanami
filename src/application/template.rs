use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use crate::protocol::event::Event;

struct TemplatePlugin;

#[async_trait]
impl super::Application for TemplatePlugin {
    fn name(&self) -> &str {
        "template"
    }
    async fn on_load(&mut self) -> Result<()> {
        log::info!("plugn <{}> loaded", self.name());
        Ok(())
    }

    async fn on_event(&mut self, event: Arc<Event>) -> Result<()> {
        todo!()
    }
}

impl TemplatePlugin {
    pub fn new() -> Self {
        Self {}
    }
}
