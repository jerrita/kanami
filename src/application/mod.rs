use std::sync::Arc;

use crate::{
    application::{
        builtin::BuiltinApp, cat::CatApp, chat::ChatApp, cron::CronApp, gscore::GSCoreAdapter,
        muri::MuriApp, ping::PingApp,
    },
    config,
    protocol::event::Event,
};
use anyhow::Result;
use async_trait::async_trait;
use lazy_static::lazy_static;
use tokio::sync::Mutex;

mod builtin;
mod cat;
mod chat;
mod gscore;
mod muri;
mod ping;

pub mod cron;

type AppType = Arc<Mutex<Box<dyn Application>>>;

// async_trait adds #[must_use] to methods returning an already must-use Future.
#[allow(clippy::double_must_use)]
#[async_trait]
pub trait Application: Send + Sync {
    fn name(&self) -> &str;
    async fn on_load(&mut self) -> Result<()> {
        log::info!("app <{}> loaded", self.name());
        Ok(())
    }
    async fn on_event(&mut self, event: Arc<Event>) -> Result<()>;
}

fn create_app(app: Box<dyn Application>) -> AppType {
    Arc::new(Mutex::new(app))
}

lazy_static! {
    pub static ref APPS: Vec<AppType> = vec![
        create_app(Box::new(BuiltinApp::new())),
        create_app(Box::new(PingApp::new())),
        create_app(Box::new(GSCoreAdapter::new())),
        create_app(Box::new(CatApp::new())),
        create_app(Box::new(CronApp::new())),
        create_app(Box::new(ChatApp::new(
            config::get().openai_token.as_str(),
            config::get().openai_base.as_str()
        ))),
        create_app(Box::new(MuriApp::new()))
    ];
}
