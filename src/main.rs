use crate::protocol::adapter;
use anyhow::Result;

mod application;
mod config;
mod logger;
mod protocol;

#[tokio::main]
async fn main() -> Result<()> {
    logger::init();
    log::info!("Hello Kanami Bot!");
    adapter::launch().await
}
