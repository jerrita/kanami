use std::sync::Arc;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use dashmap::DashMap;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Instant;

use crate::{
    config,
    protocol::{
        event::{Event, MessageEvent},
        get_bot,
        message::{Message, Segment},
    },
};

const SYSTEM_PROMPT: &str = "你是一个AI助手，名字叫 Chihaya Anon。你的回答将展示在纯文本环境中，没有markdown渲染支持，你需要基于此优化排版。你的回答需要遵守中国法律，拒绝回答任何跟政治有关的问题以及涉嫌人身霸凌的问题。若无指定，使用中文进行回答。";

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
enum ChatContent {
    Text(String),
    Parts(Vec<serde_json::Value>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ChatMessage {
    role: String,
    content: ChatContent,
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

            let mut text_prompt = String::new();
            let mut image_urls: Vec<String> = Vec::new();
            for segment in event.message() {
                match segment {
                    Segment::Text { text } => {
                        text_prompt.push_str(&text);
                    }
                    Segment::Image { url, .. } => {
                        if let Some(url) = url {
                            image_urls.push(url.clone());
                        }
                    }
                    _ => {}
                }
            }

            let (cmd, prompt) = match text_prompt.trim().split_once(' ') {
                Some((cmd, prompt)) => (cmd, prompt.trim()),
                None => (text_prompt.trim(), ""),
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
                    event
                        .reply("你问得太快了，休息一下吧~".to_string(), true)
                        .await?;
                    return Ok(());
                }

                requests.push(now);
            }

            if prompt.is_empty() && image_urls.is_empty() {
                return Ok(());
            }

            let user_content = if image_urls.is_empty() {
                ChatContent::Text(prompt.to_string())
            } else {
                let mut parts: Vec<serde_json::Value> = vec![json!({
                    "type": "text",
                    "text": prompt
                })];
                for url in image_urls {
                    parts.push(json!({
                        "type": "image_url",
                        "image_url": { "url": url }
                    }));
                }
                ChatContent::Parts(parts)
            };
            let user_message = ChatMessage {
                role: "user".to_string(),
                content: user_content,
            };

            let thinking_res = event.reply("少女思考中...".to_string(), true).await?;
            let thinking_msg_id = thinking_res
                .data
                .as_ref()
                .and_then(|d| d.get("message_id"))
                .and_then(|v| v.as_i64())
                .map(|id| id as i32)
                .ok_or_else(|| anyhow!("Failed to get message_id"))?;

            match cmd {
                "!ai" => {
                    let mut messages = vec![ChatMessage {
                        role: "system".to_string(),
                        content: ChatContent::Text(SYSTEM_PROMPT.to_string()),
                    }];
                    messages.push(user_message);
                    self.execute_chat_and_reply(event, &mut messages).await?;
                    self.history.insert(user_id, messages);
                }
                "!aip" => {
                    let mut history = self.history.entry(user_id).or_default();
                    if history.is_empty() {
                        history.push(ChatMessage {
                            role: "system".to_string(),
                            content: ChatContent::Text(SYSTEM_PROMPT.to_string()),
                        });
                    }
                    if history.len() >= 6 {
                        let summary = self.summarize_history(&history).await?;
                        *history = vec![
                            ChatMessage {
                                role: "system".to_string(),
                                content: ChatContent::Text(SYSTEM_PROMPT.to_string()),
                            },
                            ChatMessage {
                                role: "system".to_string(),
                                content: ChatContent::Text(format!(
                                    "这是之前对话的摘要: {}",
                                    summary
                                )),
                            },
                        ];
                    }
                    history.push(user_message);
                    self.execute_chat_and_reply(event, &mut history).await?;
                }
                _ => {}
            }

            get_bot().await.delete_message(thinking_msg_id).await?;
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

    async fn execute_chat_and_reply(
        &self,
        event: &MessageEvent,
        messages: &mut Vec<ChatMessage>,
    ) -> Result<()> {
        let res = self.call_api(messages).await?;
        let res_text = match res.content {
            ChatContent::Text(text) => text,
            ChatContent::Parts(_) => {
                return Err(anyhow!("Unsupported multimodal response from API"));
            }
        };

        messages.push(ChatMessage {
            role: res.role,
            content: ChatContent::Text(res_text.clone()),
        });

        self.send_reply(event, res_text).await
    }

    async fn send_reply(&self, event: &MessageEvent, text: String) -> Result<()> {
        let url_re = Regex::new(r"(https?://[\S]+\.(?:png|jpg|jpeg|gif|webp))").unwrap();
        let mut segments = Vec::new();
        let mut last_end = 0;

        for mat in url_re.find_iter(&text) {
            if mat.start() > last_end {
                let text_part = &text[last_end..mat.start()];
                segments.push(Segment::Text {
                    text: text_part.to_string(),
                });
            }

            let url = mat.as_str().to_string();
            segments.push(Segment::image(url));

            last_end = mat.end();
        }

        if last_end < text.len() {
            segments.push(Segment::Text {
                text: text[last_end..].to_string(),
            });
        }

        if segments.is_empty() {
            if !text.is_empty() {
                event.reply(text, true).await?;
            }
        } else {
            event.reply(Message::from(segments), true).await?;
        }

        Ok(())
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
            content: ChatContent::Text(
                "Summarize the following conversation history concisely.".to_string(),
            ),
        }];
        summary_messages.extend_from_slice(messages);

        let res = self.call_api(&summary_messages).await?;
        match res.content {
            ChatContent::Text(text) => Ok(text),
            _ => Err(anyhow!("Unsupported summary response from API")),
        }
    }
}
