use std::sync::Arc;

use crate::{
    application::{builtin::BuiltinApp, ping::PingApp},
    protocol::event::Event,
};
use anyhow::Result;
use async_trait::async_trait;
use lazy_static::lazy_static;
use tokio::sync::Mutex;

mod builtin;
mod ping;

type AppType = Arc<Mutex<Box<dyn Application>>>;

#[async_trait]
pub trait Application: Send + Sync {
    fn name(&self) -> &str;
    async fn on_load(&mut self) -> Result<()>;
    async fn on_event(&mut self, event: Arc<Event>) -> Result<()>;
}

fn create_app(app: Box<dyn Application>) -> AppType {
    Arc::new(Mutex::new(app))
}

lazy_static! {
    pub static ref APPS: [AppType; 2] = [
        create_app(Box::new(BuiltinApp::new())),
        create_app(Box::new(PingApp::new()))
    ];
}
