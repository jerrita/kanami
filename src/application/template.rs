use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use crate::protocol::event::Event;

struct TemplateApp;

#[async_trait]
impl super::Application for TemplateApp {
    fn name(&self) -> &str {
        "template"
    }

    async fn on_event(&mut self, event: Arc<Event>) -> Result<()> {
        todo!()
    }
}

impl TemplateApp {
    pub fn new() -> Self {
        Self {}
    }
}
