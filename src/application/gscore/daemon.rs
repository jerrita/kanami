use crate::application::gscore::model::*;
use crate::config;
use crate::protocol::get_bot;
use crate::protocol::message::Segment;
use anyhow::{Result, anyhow};
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::{
    connect_async_with_config,
    tungstenite::{Message as WsMessage, protocol::WebSocketConfig},
};

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// GSCore主循环，处理自动重连
pub async fn gscore_loop(mut receiver: mpsc::Receiver<MessageReceive>) {
    let mut retry = 0u32;
    loop {
        match connect_gscore().await {
            Ok(ws) => {
                retry = 0;
                if let Err(e) = event_loop(ws, &mut receiver).await {
                    log::error!("GSCore event loop error: {}", e);
                }
            }
            Err(e) => {
                log::warn!("GSCore connection error: {}", e);
            }
        }
        log::info!("GSCore reconnecting after 3s. times: {}", retry);
        tokio::time::sleep(Duration::from_secs(3)).await;
        retry += 1;
    }
}

async fn connect_gscore() -> Result<WsStream> {
    log::info!("Connecting to GSCore at {}", config::GSCORE_ENDPOINT);

    let mut ws_config = WebSocketConfig::default();
    ws_config.max_message_size = Some(64 * 1024 * 1024);
    ws_config.max_frame_size = Some(16 * 1024 * 1024);

    let (ws, _) =
        connect_async_with_config(config::GSCORE_ENDPOINT, Some(ws_config), false).await?;

    log::info!("GSCore WebSocket connection established");
    Ok(ws)
}

async fn event_loop(ws: WsStream, receiver: &mut mpsc::Receiver<MessageReceive>) -> Result<()> {
    let (mut ws_sender, mut ws_receiver) = ws.split();

    loop {
        tokio::select! {
            // 处理来自应用的消息
            msg = receiver.recv() => {
                match msg {
                    Some(message_receive) => {
                        let json_str = serde_json::to_string(&message_receive)?;
                        log::debug!("Sending message to GSCore: {}", json_str);
                        // GSCore 期望接收 bytes 格式，所以发送 Binary 消息而不是 Text
                        if let Err(e) = ws_sender.send(WsMessage::Binary(json_str.into())).await {
                            log::error!("Failed to send message to GSCore: {}", e);
                            return Err(anyhow!("Send error: {}", e));
                        }
                    }
                    None => {
                        log::info!("Receiver channel closed");
                        return Ok(());
                    }
                }
            }
            // 处理来自 GSCore 的消息
            ws_msg = ws_receiver.next() => {
                match ws_msg {
                    Some(Ok(WsMessage::Text(text))) => {
                        if let Err(e) = handle_gscore_message(&text).await {
                            log::error!("Failed to handle GSCore message: {}", e);
                        }
                    }
                    Some(Ok(WsMessage::Binary(data))) => {
                        match std::str::from_utf8(&data) {
                            Ok(text) => {
                                if let Err(e) = handle_gscore_message(text).await {
                                    log::error!("Failed to handle GSCore binary message: {}", e);
                                }
                            }
                            Err(e) => {
                                log::warn!("Received non-UTF8 binary message from GSCore: {}", e);
                            }
                        }
                    }
                    Some(Ok(WsMessage::Ping(payload))) => {
                        if let Err(e) = ws_sender.send(WsMessage::Pong(payload)).await {
                            log::error!("Failed to send pong: {}", e);
                            return Err(anyhow!("Pong error: {}", e));
                        }
                        log::debug!("Responded to ping from GSCore");
                    }
                    Some(Ok(WsMessage::Pong(_))) => {
                        log::debug!("Received pong from GSCore");
                    }
                    Some(Ok(WsMessage::Close(_))) => {
                        log::info!("GSCore connection closed");
                        return Ok(());
                    }
                    Some(Err(e)) => {
                        log::error!("WebSocket error: {}", e);
                        return Err(anyhow!("WebSocket error: {}", e));
                    }
                    None => {
                        log::info!("WebSocket stream ended");
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
    }
}

/// 处理来自 GSCore 的消息
async fn handle_gscore_message(text: &str) -> Result<()> {
    let msg_size = text.len();
    let preview = if text.len() > 400 { &text[..400] } else { text };
    log::debug!("Received from GSCore: {}...", preview);

    let msg_send = match serde_json::from_str::<MessageSend>(text) {
        Ok(msg) => msg,
        Err(e) => {
            log::warn!("Failed to parse GSCore message as MessageSend: {}", e);
            return Ok(()); // 直接返回，避免继续处理
        }
    };

    log::debug!("Processing MessageSend of size: {} bytes", msg_size);

    // 处理消息
    process_message_send(msg_send).await
}

async fn process_message_send(mut send: MessageSend) -> Result<()> {
    let content = match send.content.take() {
        Some(content) => content,
        None => return Ok(()), // 空内容直接返回
    };

    if let Some(GSCoreMessage::Log(msg)) = content.first() {
        log::info!("GSCore: {}", msg);
        return Ok(());
    }

    let target_type = match send.target_type.ok_or_else(|| anyhow!("no target_type"))? {
        TargetType::Group => "group",
        _ => "private",
    };
    let target: i64 = send
        .target_id
        .ok_or_else(|| anyhow!("no target_id"))?
        .parse()?;

    let forwards = content
        .iter()
        .filter_map(|x| {
            if let GSCoreMessage::Node(x) = x {
                Some(
                    x.iter()
                        .map(|e| Segment::Node {
                            id: None,
                            user_id: Some(config::GSCORE_NODE_SENDER_ID.to_string()),
                            nickname: Some(config::GSCORE_NODE_SENDER_NICKNAME.to_string()),
                            content: Some(Segment::from(e).into()),
                        })
                        .collect::<Vec<Segment>>(),
                )
            } else {
                None
            }
        })
        .flatten()
        .collect::<Vec<Segment>>();

    if !forwards.is_empty() {
        get_bot()
            .await
            .send_forward_msg(Some(target_type), Some(target), Some(target), forwards)
            .await?;
    } else {
        get_bot()
            .await
            .send_message(Some(target_type), Some(target), Some(target), content)
            .await?;
    }
    Ok(())
}
