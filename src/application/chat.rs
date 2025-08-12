use std::sync::Arc;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use dashmap::DashMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

use crate::{
    config,
    protocol::{
        event::{Event, MessageEvent},
        get_bot,
    },
};

const SYSTEM_PROMPT: &str = "你是一个AI助手，名字叫 Chihaya Anon。你的回答需要遵守中国法律，拒绝回答任何跟政治有关的问题以及涉嫌人身霸凌的问题。若无指定，使用中文进行回答。";

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessage,
}

pub struct ChatApp {
    client: Client,
    token: String,
    base_url: String,
    history: DashMap<i64, Vec<ChatMessage>>,
    rate_limiter: DashMap<i64, Vec<Instant>>,
}

#[async_trait]
impl super::Application for ChatApp {
    fn name(&self) -> &str {
        "chat"
    }

    async fn on_event(&mut self, event: Arc<Event>) -> Result<()> {
        if let Event::MessageEvent(event) = event.as_ref() {
            if let MessageEvent::Group(g) = event {
                if !config::WHITE_GROUPS.contains(&g.group_id) {
                    return Ok(());
                }
            }

            let msg = event.raw_message();
            let (cmd, prompt) = match msg.split_once(' ') {
                Some((cmd, prompt)) => (cmd, prompt.trim()),
                None => return Ok(()),
            };

            if !matches!(cmd, "!ai" | "!aip") {
                return Ok(());
            }

            let user_id = event.user_id();

            {
                let now = Instant::now();
                let mut requests = self.rate_limiter.entry(user_id).or_default();
                requests.retain(|&t| now.duration_since(t).as_secs() < 60);

                if requests.len() >= 3 {
                    event.reply("你问得太快了，休息一下吧~", true).await?;
                    return Ok(());
                }

                requests.push(now);
            }

            if prompt.is_empty() {
                return Ok(());
            }

            match cmd {
                "!ai" => {
                    let thinking_res = event.reply("少女思考中...", true).await?;
                    let thinking_msg_id = thinking_res
                        .data
                        .as_ref()
                        .and_then(|d| d.get("message_id"))
                        .and_then(|v| v.as_i64())
                        .map(|id| id as i32)
                        .ok_or_else(|| anyhow!("Failed to get message_id"))?;

                    let mut messages = vec![
                        ChatMessage {
                            role: "system".to_string(),
                            content: SYSTEM_PROMPT.to_string(),
                        },
                        ChatMessage {
                            role: "user".to_string(),
                            content: prompt.to_string(),
                        },
                    ];
                    let res = self.call_api(&messages).await?;
                    messages.push(res.clone());
                    self.history.insert(user_id, messages);
                    event.reply(res.content, true).await?;
                    get_bot().await.delete_message(thinking_msg_id).await?;
                }
                "!aip" => {
                    let thinking_res = event.reply("少女思考中...", true).await?;
                    let thinking_msg_id = thinking_res
                        .data
                        .as_ref()
                        .and_then(|d| d.get("message_id"))
                        .and_then(|v| v.as_i64())
                        .map(|id| id as i32)
                        .ok_or_else(|| anyhow!("Failed to get message_id"))?;

                    let mut history = self.history.entry(user_id).or_default();
                    if history.is_empty() {
                        history.push(ChatMessage {
                            role: "system".to_string(),
                            content: SYSTEM_PROMPT.to_string(),
                        });
                    }

                    if history.len() >= 6 {
                        let summary = self.summarize_history(&history).await?;
                        *history = vec![
                            ChatMessage {
                                role: "system".to_string(),
                                content: SYSTEM_PROMPT.to_string(),
                            },
                            ChatMessage {
                                role: "system".to_string(),
                                content: format!("这是之前对话的摘要: {}", summary),
                            },
                        ];
                    }
                    history.push(ChatMessage {
                        role: "user".to_string(),
                        content: prompt.to_string(),
                    });
                    let res = self.call_api(&history).await?;
                    history.push(res.clone());
                    event.reply(res.content, true).await?;
                    get_bot().await.delete_message(thinking_msg_id).await?;
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl ChatApp {
    pub fn new(token: &str, base_url: &str) -> Self {
        Self {
            client: Client::new(),
            token: token.to_string(),
            base_url: base_url.to_string(),
            history: DashMap::new(),
            rate_limiter: DashMap::new(),
        }
    }

    async fn call_api(&self, messages: &[ChatMessage]) -> Result<ChatMessage> {
        let req = ChatRequest {
            model: "gpt-4.1",
            messages,
        };

        let res = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .bearer_auth(&self.token)
            .json(&req)
            .send()
            .await?
            .json::<ChatResponse>()
            .await?;

        res.choices
            .into_iter()
            .next()
            .map(|c| c.message)
            .ok_or_else(|| anyhow!("API response is empty"))
    }

    async fn summarize_history(&self, messages: &[ChatMessage]) -> Result<String> {
        let mut summary_messages = vec![ChatMessage {
            role: "system".to_string(),
            content: "Summarize the following conversation history concisely.".to_string(),
        }];
        summary_messages.extend_from_slice(messages);

        self.call_api(&summary_messages).await.map(|m| m.content)
    }
}
