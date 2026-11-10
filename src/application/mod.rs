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
use serde::de::DeserializeOwned;
use tokio::sync::Mutex;

pub(crate) mod builtin;
pub(crate) mod cat;
pub(crate) mod chat;
pub(crate) mod gscore;
pub(crate) mod muri;
pub(crate) mod ping;

pub mod cron;

pub type AppType = Arc<Mutex<Box<dyn MountedApp>>>;

// async_trait adds #[must_use] to methods returning an already must-use Future.
#[allow(clippy::double_must_use)]
#[async_trait]
pub trait Application: Send + Sync + Sized + 'static {
    const NAME: &'static str;
    type Config: DeserializeOwned;

    fn new(config: Self::Config) -> Result<Self>;
    async fn start(&mut self) -> Result<()> {
        Ok(())
    }
    async fn on_event(&mut self, event: Arc<Event>) -> Result<()>;
}

#[async_trait]
pub trait MountedApp: Send + Sync {
    fn name(&self) -> &'static str;
    async fn start(&mut self) -> Result<()>;
    async fn on_event(&mut self, event: Arc<Event>) -> Result<()>;
}

struct Mounted<A>(A);

#[async_trait]
impl<A: Application> MountedApp for Mounted<A> {
    fn name(&self) -> &'static str {
        A::NAME
    }

    async fn start(&mut self) -> Result<()> {
        self.0.start().await
    }

    async fn on_event(&mut self, event: Arc<Event>) -> Result<()> {
        self.0.on_event(event).await
    }
}

fn create_app<A: Application>(config: A::Config) -> Result<AppType> {
    Ok(Arc::new(Mutex::new(Box::new(Mounted(A::new(config)?)))))
}

pub async fn start() -> Result<Vec<AppType>> {
    let config = config::get();
    let mut apps = Vec::new();
    if let Some(config) = &config.builtin {
        apps.push(create_app::<BuiltinApp>(config.clone())?);
    }
    if let Some(config) = &config.ping {
        apps.push(create_app::<PingApp>(config.clone())?);
    }
    if let Some(config) = &config.gscore {
        apps.push(create_app::<GSCoreAdapter>(config.clone())?);
    }
    if let Some(config) = &config.cat {
        apps.push(create_app::<CatApp>(config.clone())?);
    }
    if let Some(config) = &config.cron {
        apps.push(create_app::<CronApp>(config.clone())?);
    }
    if let Some(config) = &config.chat {
        apps.push(create_app::<ChatApp>(config.clone())?);
    }
    if let Some(config) = &config.muri {
        apps.push(create_app::<MuriApp>(config.clone())?);
    }
    for app in &apps {
        let mut app = app.lock().await;
        app.start().await?;
        log::info!("app <{}> started", app.name());
    }
    Ok(apps)
}
