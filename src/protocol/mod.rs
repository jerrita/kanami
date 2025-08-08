use crate::protocol::adapter::{Request, Response};
use anyhow::{Result, anyhow};
use futures_util::lock::Mutex;
use lazy_static::lazy_static;
use serde_json::Value;
use tokio::sync::{mpsc::UnboundedSender, oneshot};
use uuid::Uuid;

pub mod event;
pub mod message;

pub mod adapter;
mod extension;

type RequestSender = UnboundedSender<Request>;

#[derive(Clone)]
pub struct Protocol {
    sender: Option<RequestSender>,
}

impl Protocol {
    async fn send_request(&self, func: &str, data: Value) -> Result<Response> {
        let (tx, rx) = oneshot::channel();
        let request = Request {
            action: func.to_string(),
            params: data,
            echo: Uuid::new_v4().to_string(),
            created_at: std::time::Instant::now(),
            sender: tx,
        };

        self.sender
            .clone()
            .ok_or(anyhow!("sender not found"))?
            .send(request)?;
        Ok(rx.await?)
    }
}

lazy_static! {
    pub static ref BOT: Mutex<Protocol> = Mutex::new(Protocol { sender: None });
}

pub async fn get_bot() -> Protocol {
    BOT.lock().await.clone()
}

pub async fn update(sender: RequestSender) {
    *BOT.lock().await = Protocol {
        sender: Some(sender),
    };
}
