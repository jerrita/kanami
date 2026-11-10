use std::sync::Arc;

use crate::{
    application::{
        builtin::BuiltinApp, cat::CatApp, chat::ChatApp, cron::CronApp, gscore::GSCoreAdapter,
        muri::MuriApp, ping::PingApp,
    },
    config,
    protocol::event::Event,
};
use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use serde::{Deserialize, de::DeserializeOwned};
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

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum List {
    Allow(Vec<i64>),
    Deny(Vec<i64>),
}

impl List {
    fn allows(&self, id: i64) -> bool {
        match self {
            Self::Allow(ids) => ids.contains(&id),
            Self::Deny(ids) => !ids.contains(&id),
        }
    }
}

struct Mounted<A> {
    app: A,
    groups: Option<List>,
    users: Option<List>,
}

#[async_trait]
impl<A: Application> MountedApp for Mounted<A> {
    fn name(&self) -> &'static str {
        A::NAME
    }

    async fn start(&mut self) -> Result<()> {
        self.app.start().await
    }

    async fn on_event(&mut self, event: Arc<Event>) -> Result<()> {
        if let (Some(groups), Some(id)) = (&self.groups, event.group_id())
            && !groups.allows(id)
        {
            return Ok(());
        }
        if let (Some(users), Some(id)) = (&self.users, event.user_id())
            && !users.allows(id)
        {
            return Ok(());
        }
        self.app.on_event(event).await
    }
}

struct Registration {
    name: &'static str,
    mount: fn(toml::Table) -> Result<AppType>,
}

fn mount<A: Application>() -> Registration {
    Registration {
        name: A::NAME,
        mount: |mut table| {
            let groups = table
                .remove("groups")
                .map(toml::Value::try_into)
                .transpose()
                .context("groups")?;
            let users = table
                .remove("users")
                .map(toml::Value::try_into)
                .transpose()
                .context("users")?;
            let config = table.try_into::<A::Config>()?;
            Ok(Arc::new(Mutex::new(Box::new(Mounted {
                app: A::new(config)?,
                groups,
                users,
            }))))
        },
    }
}

pub fn create(config: &config::Config) -> Result<Vec<AppType>> {
    let registry = [
        mount::<BuiltinApp>(),
        mount::<PingApp>(),
        mount::<GSCoreAdapter>(),
        mount::<CatApp>(),
        mount::<CronApp>(),
        mount::<ChatApp>(),
        mount::<MuriApp>(),
    ];
    for name in config.app.keys() {
        if !registry.iter().any(|app| app.name == name) {
            bail!("unknown app: {name}");
        }
    }
    config
        .apps
        .iter()
        .map(|name| {
            let registration = registry
                .iter()
                .find(|app| app.name == name)
                .with_context(|| format!("unknown app: {name}"))?;
            let table = match config.app.get(name) {
                Some(table) => table.clone(),
                None => toml::Table::new(),
            };
            (registration.mount)(table).with_context(|| format!("app.{name}"))
        })
        .collect()
}

pub async fn start() -> Result<Vec<AppType>> {
    let apps = create(config::get())?;
    for app in &apps {
        let mut app = app.lock().await;
        app.start()
            .await
            .with_context(|| format!("app.{} start", app.name()))?;
        log::info!("app <{}> started", app.name());
    }
    Ok(apps)
}
