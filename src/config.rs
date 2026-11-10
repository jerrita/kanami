use std::{collections::BTreeMap, env, fs, path::PathBuf, sync::OnceLock};

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Deserializer, de::Error};

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Alias {
    #[serde(default)]
    pub group: BTreeMap<String, i64>,
    #[serde(default)]
    pub user: BTreeMap<String, i64>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Id {
    Number(i64),
    Alias(String),
}

impl Id {
    pub fn resolve(self, aliases: &BTreeMap<String, i64>) -> Result<i64> {
        match self {
            Self::Number(id) => Ok(id),
            Self::Alias(name) => aliases
                .get(&name)
                .copied()
                .with_context(|| format!("undefined alias: {name}")),
        }
    }
}

#[derive(Deserialize)]
#[serde(try_from = "RawConfig")]
pub struct Config {
    pub endpoint: String,
    pub token: String,
    pub owner: i64,
    pub apps: Vec<String>,
    pub alias: Alias,
    pub app: BTreeMap<String, toml::Table>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    endpoint: String,
    token: String,
    owner: Id,
    apps: Vec<String>,
    #[serde(default)]
    alias: Alias,
    #[serde(default)]
    app: BTreeMap<String, toml::Table>,
}

impl TryFrom<RawConfig> for Config {
    type Error = anyhow::Error;

    fn try_from(raw: RawConfig) -> Result<Self> {
        let owner = raw
            .owner
            .resolve(&raw.alias.user)
            .map_err(|error| anyhow!("owner: {error}"))?;
        Ok(Self {
            endpoint: raw.endpoint,
            token: raw.token,
            owner,
            apps: raw.apps,
            alias: raw.alias,
            app: raw.app,
        })
    }
}

// App configs are deserialized after the system config has been loaded.
pub fn group_id<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i64, D::Error> {
    Id::deserialize(deserializer)?
        .resolve(&get().alias.group)
        .map_err(D::Error::custom)
}

pub fn user_id<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i64, D::Error> {
    Id::deserialize(deserializer)?
        .resolve(&get().alias.user)
        .map_err(D::Error::custom)
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn load() -> Result<()> {
    let path = match env::var_os("KANAMI_CONFIG") {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from("config.toml"),
    };
    let source = fs::read_to_string(&path)
        .with_context(|| format!("failed to read config {}", path.display()))?;
    let config = toml::from_str::<Config>(&source)
        .map_err(|mut error| {
            error.set_input(None);
            error
        })
        .with_context(|| format!("invalid config {}", path.display()))?;
    CONFIG
        .set(config)
        .map_err(|_| anyhow!("config already loaded"))
}

pub fn get() -> &'static Config {
    CONFIG
        .get()
        .expect("config must be loaded before starting applications")
}

#[cfg(test)]
mod tests {
    use super::{CONFIG, Config};

    #[test]
    fn example_matches_schema() {
        let config = toml::from_str::<Config>(include_str!("../config.example.toml")).unwrap();
        assert!(CONFIG.set(config).is_ok());
        crate::application::create().unwrap();
    }
}
