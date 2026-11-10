use crate::protocol::adapter;
use anyhow::Result;
// use tikv_jemallocator::Jemalloc;

mod application;
mod config;
mod logger;
mod protocol;

// #[global_allocator]
// static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() -> Result<()> {
    config::load()?;
    logger::init();
    log::info!("Hello Kanami Bot!");
    let apps = application::start().await?;
    adapter::launch(apps).await
}
