// to connect to a ws backend
use crate::{application::APPS, config, protocol::event::Event};
use anyhow::{Result, anyhow};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt, stream::SplitStream};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{connect_async, tungstenite::Message};

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

#[derive(Deserialize, Debug)]
#[allow(unused)]
pub struct Response {
    status: String,
    retcode: i32,
    pub data: Value,
    message: Option<String>,
    echo: Option<String>,
}

#[derive(Serialize)]
pub struct Request {
    pub action: String,
    pub params: Value,
    pub echo: String,

    #[serde(skip_serializing)]
    pub sender: oneshot::Sender<Response>,
}

pub async fn listener(
    pending_requests: Arc<DashMap<String, oneshot::Sender<Response>>>,
    mut receiver: SplitStream<WsStream>,
) {
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                log::debug!("{}", text);
                if let Ok(raw) = serde_json::from_str::<Value>(&text) {
                    if let Some(echo) = raw.get("echo").and_then(|v| v.as_str()) {
                        if let Some((_, sender)) = pending_requests.remove(echo) {
                            log::debug!("Session resume: {}", echo);
                            match serde_json::from_value::<Response>(raw) {
                                Ok(res) => _ = sender.send(res),
                                Err(e) => log::warn!("resp parse error: {}", e),
                            }
                        } else {
                            log::warn!("Message received with unknown UUID: {}", echo);
                        }
                    } else {
                        let event = serde_json::from_value::<Event>(raw);
                        match event {
                            Ok(event) => {
                                let event = Arc::new(event);
                                for app in APPS.iter() {
                                    let event = event.clone();
                                    tokio::spawn(async move {
                                        let mut app = app.lock().await;
                                        if let Err(e) = app.on_event(event).await {
                                            log::warn!("app <{}> process error: {}", app.name(), e)
                                        }
                                    });
                                }
                            }
                            Err(e) => log::warn!("deserialize error: {}", e),
                        };
                    }
                }
            }
            Ok(Message::Close(_)) => {
                log::warn!("connection closed.");
                break;
            }
            Err(e) => {
                log::error!("listener: failed with error: {}", e);
                break;
            }
            _ => log::warn!("unknown msg type: {:?}", unsafe { msg.unwrap_unchecked() }),
        }
    }
}

async fn connect() -> Result<WsStream> {
    log::info!("=> {}", config::ENDPOINT);
    let url = format!("{}?access_token={}", config::ENDPOINT, config::TOKEN);
    let (mut ws, _) = connect_async(&url).await?;
    let msg = ws.next().await.ok_or(anyhow!("WebSocket stream ended"))??;
    let text = match msg {
        Message::Text(text) => text,
        _ => return Err(anyhow!("Expected text message, check your endpoint")),
    };
    log::debug!("connect text: {}", text);
    let value = serde_json::from_str::<Value>(&text)?;
    if value.get("echo").is_some() {
        let res = serde_json::from_value::<Response>(value)?;
        if res.retcode != 200 {
            Err(anyhow!(format!("{:?}", res)))?
        }
    } else {
        log::info!("Bot {} conncted!", value.get("self_id").unwrap_or_default());
    }
    Ok(ws)
}

async fn event_loop(ws: WsStream) -> Result<()> {
    let (mut ws_sender, ws_receiver) = ws.split();
    let (req_tx, mut req_rx) = mpsc::unbounded_channel();

    super::update(req_tx).await;
    let pending_requests = Arc::new(DashMap::new());
    let pending_requests_cloned = pending_requests.clone();

    let task_event_listener = tokio::spawn(async move {
        listener(pending_requests_cloned, ws_receiver).await;
    });
    let task_sender = tokio::spawn(async move {
        while let Some(request) = req_rx.recv().await {
            log::debug!("request: {:?} -> {}", request.action, request.params);
            if let Ok(message_str) = serde_json::to_string(&request) {
                let message = Message::from(message_str);
                pending_requests.insert(request.echo.clone(), request.sender);
                log::debug!("Session {} created", request.echo);
                if let Err(e) = ws_sender.send(message).await {
                    log::error!("RequestTask: failed with error: {}", e);
                }
            } else {
                log::error!("failed to serialize request");
            }
        }
    });

    for app in APPS.iter() {
        tokio::spawn(async move {
            let mut app = app.lock().await;
            if let Err(e) = app.on_load().await {
                log::warn!("app <{}> on_load error: {}", app.name(), e)
            }
        });
    }

    tokio::select! {
        _ = task_event_listener => {
            log::info!("Listener task endded");
        }
        _ = task_sender => {
            log::info!("Sender task endded");
        }
    }
    Ok(())
}

pub async fn launch() -> ! {
    let mut retry: u32 = 0;
    loop {
        match connect().await {
            Ok(ws) => {
                if let Err(e) = event_loop(ws).await {
                    log::error!("loop error: {}", e);
                }
            }
            Err(e) => log::warn!("launch error: {}", e),
        }
        log::info!("reconnecting after 3s. times: {}", retry);
        tokio::time::sleep(Duration::from_secs(3)).await;
        retry += 1;
    }
}
