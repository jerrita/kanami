use std::sync::Arc;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use dashmap::DashMap;
use log::{debug, error};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::RwLock;
use tokio::time::Instant;

use crate::{
    config,
    protocol::{
        event::{Event, MessageEvent},
        message::{Message, Segment},
    },
};

const SYSTEM_PROMPT: &str = "你是一个AI助手，名字叫 Chihaya Anon。你的回答需要遵守中国法律，拒绝回答任何跟政治有关的问题以及涉嫌人身霸凌的问题。若无指定，使用中文进行回答。你的回答为无代码块包裹的rst格式。不要使用粗体和斜体，除非你有充分的理由那么做。";
const RATE_LIMIT_WINDOW: u64 = 60; // seconds
const RATE_LIMIT_MAX: usize = 3;
const HISTORY_MAX_LENGTH: usize = 6;
const MAX_MESSAGE_LENGTH: usize = 2800;

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

// Helper struct to reduce parameter passing
struct ChatContext {
    client: Client,
    token: String,
    base_url: String,
    current_model: Arc<RwLock<String>>,
}

impl ChatContext {
    fn new(
        client: Client,
        token: String,
        base_url: String,
        current_model: Arc<RwLock<String>>,
    ) -> Self {
        Self {
            client,
            token,
            base_url,
            current_model,
        }
    }
}

pub struct ChatApp {
    client: Client,
    token: String,
    base_url: String,
    current_model: Arc<RwLock<String>>,
    history: DashMap<i64, Vec<ChatMessage>>,
    rate_limiter: DashMap<i64, Vec<Instant>>,
}

#[async_trait]
impl super::Application for ChatApp {
    fn name(&self) -> &str {
        "chat"
    }

    async fn on_event(&mut self, event: Arc<Event>) -> Result<()> {
        // Create context for concurrent processing
        let context = ChatContext::new(
            self.client.clone(),
            self.token.clone(),
            self.base_url.clone(),
            Arc::clone(&self.current_model),
        );
        let history = self.history.clone();
        let rate_limiter = self.rate_limiter.clone();

        // Spawn a task for concurrent processing
        tokio::spawn(async move {
            if let Err(e) = Self::handle_event_impl(context, history, rate_limiter, event).await {
                log::error!("Error handling chat event: {}", e);
            }
        });

        Ok(())
    }
}

impl ChatApp {
    async fn handle_event_impl(
        context: ChatContext,
        history: DashMap<i64, Vec<ChatMessage>>,
        rate_limiter: DashMap<i64, Vec<Instant>>,
        event: Arc<Event>,
    ) -> Result<()> {
        if let Event::MessageEvent(event) = event.as_ref() {
            if let MessageEvent::Group(g) = event {
                if !config::WHITE_GROUPS.contains(&g.group_id) {
                    return Ok(());
                }
            }

            let (text_prompt, images_to_process) =
                Self::extract_text_and_images(event.message().segments());
            let (cmd, prompt) = Self::parse_command(&text_prompt);

            if !matches!(cmd, "!ai" | "!aip" | "!switch") {
                return Ok(());
            }

            let user_id = event.user_id();

            // Handle !switch command separately for OWNER only
            if cmd == "!switch" {
                if user_id != config::OWNER {
                    return Ok(());
                }

                if prompt.is_empty() {
                    // Show current model when no parameter is provided
                    let current_model = context.current_model.read().await.clone();
                    event
                        .reply(
                            format!("当前模型: {}，输入模型名称以切换", current_model),
                            true,
                        )
                        .await?;
                    return Ok(());
                }

                {
                    let mut model = context.current_model.write().await;
                    *model = prompt.to_string();
                }

                event
                    .reply(format!("已切换模型为: {}", prompt), true)
                    .await?;
                return Ok(());
            }

            // Check rate limit
            if let Err(msg) = Self::check_rate_limit(&rate_limiter, user_id).await {
                event.reply(msg, true).await?;
                return Ok(());
            }

            if prompt.is_empty() && images_to_process.is_empty() {
                return Ok(());
            }

            let mut data_urls = Vec::new();
            for (file_name, url) in images_to_process {
                let response = context.client.get(&url).send().await?;
                let image_bytes = response.bytes().await?;
                let mime_type = Self::get_mime_type(&file_name);
                let encoded_image = general_purpose::STANDARD.encode(&image_bytes);
                let data_url = format!("data:{};base64,{}", mime_type, encoded_image);
                data_urls.push(data_url);
            }

            let user_content = Self::create_user_content(prompt, data_urls);
            let user_message = ChatMessage {
                role: "user".to_string(),
                content: user_content,
            };

            match cmd {
                "!ai" => {
                    let mut messages = vec![Self::create_system_message()];
                    messages.push(user_message);
                    Self::execute_chat_and_reply(&context, event, &mut messages).await?;
                    history.insert(user_id, messages);
                }
                "!aip" => {
                    let mut hist = history.entry(user_id).or_default();
                    if hist.is_empty() {
                        hist.push(Self::create_system_message());
                    }
                    if hist.len() >= HISTORY_MAX_LENGTH {
                        let summary = Self::summarize_history(&context, &hist).await?;
                        *hist = vec![
                            Self::create_system_message(),
                            ChatMessage {
                                role: "system".to_string(),
                                content: ChatContent::Text(format!(
                                    "这是之前对话的摘要: {}",
                                    summary
                                )),
                            },
                        ];
                    }
                    hist.push(user_message);
                    Self::execute_chat_and_reply(&context, event, &mut hist).await?;
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
            current_model: Arc::new(RwLock::new("gpt-4.1".to_string())),
            history: DashMap::new(),
            rate_limiter: DashMap::new(),
        }
    }

    fn create_system_message() -> ChatMessage {
        ChatMessage {
            role: "system".to_string(),
            content: ChatContent::Text(SYSTEM_PROMPT.to_string()),
        }
    }

    fn extract_text_and_images(segments: &[Segment]) -> (String, Vec<(String, String)>) {
        let mut text_prompt = String::new();
        let mut images_to_process = Vec::new();

        for segment in segments {
            match segment {
                Segment::Text { text } => {
                    text_prompt.push_str(text);
                }
                Segment::Image { file, url, .. } => {
                    if let Some(url) = url {
                        images_to_process.push((file.clone(), url.clone()));
                    }
                }
                _ => {}
            }
        }

        (text_prompt, images_to_process)
    }

    fn parse_command(text: &str) -> (&str, &str) {
        match text.trim().split_once(' ') {
            Some((cmd, prompt)) => (cmd, prompt.trim()),
            None => (text.trim(), ""),
        }
    }

    fn get_mime_type(file_name: &str) -> &'static str {
        match file_name.split('.').last() {
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("webp") => "image/webp",
            _ => "application/octet-stream",
        }
    }

    async fn check_rate_limit(
        rate_limiter: &DashMap<i64, Vec<Instant>>,
        user_id: i64,
    ) -> Result<(), String> {
        let now = Instant::now();
        let mut requests = rate_limiter.entry(user_id).or_default();
        requests.retain(|&t| now.duration_since(t).as_secs() < RATE_LIMIT_WINDOW);

        if requests.len() >= RATE_LIMIT_MAX {
            return Err("你问得太快了，休息一下吧~".to_string());
        }

        requests.push(now);
        Ok(())
    }

    fn create_user_content(prompt: &str, data_urls: Vec<String>) -> ChatContent {
        if data_urls.is_empty() {
            ChatContent::Text(prompt.to_string())
        } else {
            let mut parts: Vec<serde_json::Value> = vec![json!({
                "type": "text",
                "text": prompt
            })];
            for url in data_urls {
                parts.push(json!({
                    "type": "image_url",
                    "image_url": {
                        "url": url
                    }
                }));
            }
            ChatContent::Parts(parts)
        }
    }

    async fn execute_chat_and_reply(
        context: &ChatContext,
        event: &MessageEvent,
        messages: &mut Vec<ChatMessage>,
    ) -> Result<()> {
        let res = Self::call_api(context, messages).await?;
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

        Self::send_reply(event, res_text).await
    }

    fn split_text_by_length(text: &str, max_length: usize) -> Vec<String> {
        if text.len() <= max_length {
            return vec![text.to_string()];
        }

        let mut chunks = Vec::new();
        let mut start = 0;

        while start < text.len() {
            let end = if start + max_length >= text.len() {
                text.len()
            } else {
                // Try to find a good breaking point (sentence, paragraph, or space)
                let search_end = start + max_length;
                let chunk = &text[start..search_end];
                
                // Look for sentence endings first
                if let Some(pos) = chunk.rfind(&['.', '!', '?', '。', '！', '？'][..]) {
                    start + pos + 1
                } 
                // Look for paragraph breaks
                else if let Some(pos) = chunk.rfind('\n') {
                    start + pos + 1
                }
                // Look for spaces
                else if let Some(pos) = chunk.rfind(' ') {
                    start + pos + 1
                }
                // If no good breaking point, just cut at max_length
                else {
                    search_end
                }
            };

            chunks.push(text[start..end].trim().to_string());
            start = end;
        }

        chunks.into_iter().filter(|s| !s.is_empty()).collect()
    }

    async fn send_reply(event: &MessageEvent, text: String) -> Result<()> {
        // First, split the text into chunks if it's too long
        let text_chunks = Self::split_text_by_length(&text, MAX_MESSAGE_LENGTH);
        
        for (index, chunk) in text_chunks.iter().enumerate() {
            // For each chunk, process it for images and send
            let url_re = Regex::new(r"(https?://[\S]+\.(?:png|jpg|jpeg|gif|webp))").unwrap();
            let mut segments = Vec::new();
            let mut last_end = 0;

            for mat in url_re.find_iter(chunk) {
                if mat.start() > last_end {
                    let text_part = &chunk[last_end..mat.start()];
                    segments.push(Segment::Text {
                        text: text_part.to_string(),
                    });
                }

                let url = mat.as_str().to_string();
                segments.push(Segment::image(url));

                last_end = mat.end();
            }

            if last_end < chunk.len() {
                segments.push(Segment::Text {
                    text: chunk[last_end..].to_string(),
                });
            }

            // Send the chunk
            if segments.is_empty() {
                if !chunk.is_empty() {
                    event.reply(chunk.clone(), true).await?;
                }
            } else {
                event.reply(Message::from(segments), true).await?;
            }

            // Add a small delay between messages to avoid rate limiting
            if index < text_chunks.len() - 1 {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }

        Ok(())
    }

    async fn call_api(context: &ChatContext, messages: &[ChatMessage]) -> Result<ChatMessage> {
        let current_model = context.current_model.read().await.clone();

        let req = ChatRequest {
            model: &current_model,
            messages,
        };

        let request_body_json_full = serde_json::to_string(&req).unwrap_or_else(|e| e.to_string());
        let base64_re = Regex::new(r"data:image/[^;]+;base64,([a-zA-Z0-9+/=]{50,})").unwrap();
        let sanitized_body = base64_re.replace_all(
            &request_body_json_full,
            "data:image/...;base64,<base64_data>",
        );
        debug!(">>> LLM Request: {} >>>", sanitized_body);

        let response = context
            .client
            .post(format!("{}/v1/chat/completions", context.base_url))
            .bearer_auth(&context.token)
            .json(&req)
            .send()
            .await?;

        let response_text = response.text().await?;
        debug!("<<< LLM Response: {} >>>", response_text);

        let res: ChatResponse = match serde_json::from_str(&response_text) {
            Ok(r) => r,
            Err(e) => {
                error!(
                    "Failed to decode LLM response: {}. Response body: {}",
                    e, response_text
                );
                return Err(anyhow!("Failed to decode LLM response"));
            }
        };

        res.choices
            .into_iter()
            .next()
            .map(|c| c.message)
            .ok_or_else(|| anyhow!("API response is empty"))
    }

    async fn summarize_history(context: &ChatContext, messages: &[ChatMessage]) -> Result<String> {
        let mut summary_messages = vec![ChatMessage {
            role: "system".to_string(),
            content: ChatContent::Text(
                "Summarize the following conversation history concisely.".to_string(),
            ),
        }];
        summary_messages.extend_from_slice(messages);

        let res = Self::call_api(context, &summary_messages).await?;
        match res.content {
            ChatContent::Text(text) => Ok(text),
            _ => Err(anyhow!("Unsupported summary response from API")),
        }
    }
}
