use std::{env, fs, path::PathBuf, sync::OnceLock};

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

use crate::application::{builtin, cat, cron, gscore, ping};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub endpoint: String,
    pub token: String,
    pub owner: i64,
    pub main_group: i64,
    pub white_groups: Vec<i64>,
    pub openai_token: String,
    pub openai_base: String,
    pub builtin: Option<builtin::Config>,
    pub ping: Option<ping::Config>,
    pub gscore: Option<gscore::Config>,
    pub cat: Option<cat::Config>,
    pub cron: Option<cron::Config>,
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
    use super::Config;

    #[test]
    fn example_matches_schema() {
        toml::from_str::<Config>(include_str!("../config.example.toml")).unwrap();
    }
}
