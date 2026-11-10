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
        get_bot,
        message::{Message, Segment},
    },
};

const SUMMARY_PROMPT: &str = "Summarize the following conversation history concisely.";
const SUMMARY_PREFIX: &str = "这是之前对话的摘要: ";
const RATE_LIMIT_MESSAGE: &str = "你问得太快了，休息一下吧~";
const REPLY_DELAY_MS: u64 = 100;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub token: String,
    pub base_url: String,
    pub default_model: String,
    pub system_prompt: String,
    pub white_groups: Vec<i64>,
    pub rate_limit_window_secs: u64,
    pub rate_limit_max: usize,
    pub history_max_length: usize,
    pub max_message_length: std::num::NonZeroUsize,
}

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
    config: &'static Config,
    current_model: Arc<RwLock<String>>,
}

pub struct ChatApp {
    client: Client,
    config: &'static Config,
    current_model: Arc<RwLock<String>>,
    history: Arc<DashMap<i64, Vec<ChatMessage>>>,
    rate_limiter: Arc<DashMap<i64, Vec<Instant>>>,
}

#[async_trait]
impl super::Application for ChatApp {
    fn name(&self) -> &str {
        "chat"
    }

    async fn on_event(&mut self, event: Arc<Event>) -> Result<()> {
        // Create context for concurrent processing
        let context = ChatContext {
            client: self.client.clone(),
            config: self.config,
            current_model: Arc::clone(&self.current_model),
        };
        let history = Arc::clone(&self.history);
        let rate_limiter = Arc::clone(&self.rate_limiter);

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
        history: Arc<DashMap<i64, Vec<ChatMessage>>>,
        rate_limiter: Arc<DashMap<i64, Vec<Instant>>>,
        event: Arc<Event>,
    ) -> Result<()> {
        if let Event::MessageEvent(event) = event.as_ref() {
            if let MessageEvent::Group(g) = event
                && !context.config.white_groups.contains(&g.group_id)
            {
                return Ok(());
            }

            let segments = event.message().segments();
            let (mut text_prompt, mut images_to_process) = Self::extract_text_and_images(segments);

            // rework if msg contains reply
            if let Some(Segment::Reply { id }) = segments.first() {
                let bot = get_bot().await;
                if let Some(data) = bot.get_message(id.parse()?).await?.data {
                    let msg: Vec<Segment> =
                        serde_json::from_value(data.get("message").unwrap().clone())?;
                    let (a, mut b) = Self::extract_text_and_images(&msg);
                    text_prompt.push_str(&a);
                    images_to_process.append(&mut b);
                }
            }

            let (cmd, prompt) = Self::parse_command(&text_prompt);
            if !matches!(cmd, "!ai" | "!aip" | "!switch") {
                return Ok(());
            }

            debug!(
                "Message received - user_id: {}, command: '{}', prompt: '{}', images: {}",
                event.user_id(),
                cmd,
                prompt,
                images_to_process.len()
            );

            let user_id = event.user_id();

            // Handle !switch command separately for OWNER only
            if cmd == "!switch" {
                if user_id != config::get().owner {
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
            if let Err(msg) = Self::check_rate_limit(&rate_limiter, user_id, context.config).await {
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
                    let mut messages = vec![Self::create_system_message(context.config)];
                    messages.push(user_message);
                    Self::execute_chat_and_reply(&context, event, &mut messages).await?;
                    history.insert(user_id, messages);
                }
                "!aip" => {
                    let mut hist = history.entry(user_id).or_default();
                    debug!(
                        "!aip command - user_id: {}, current history length: {}",
                        user_id,
                        hist.len()
                    );
                    if hist.is_empty() {
                        hist.push(Self::create_system_message(context.config));
                        debug!("!aip - initialized empty history with system message");
                    }
                    if hist.len() >= context.config.history_max_length {
                        debug!(
                            "!aip - history length ({}) >= max length ({}), summarizing history",
                            hist.len(),
                            context.config.history_max_length
                        );
                        let summary = Self::summarize_history(&context, &hist).await?;
                        debug!("!aip - generated summary: {}", summary);
                        *hist = vec![
                            Self::create_system_message(context.config),
                            ChatMessage {
                                role: "system".to_string(),
                                content: ChatContent::Text(format!(
                                    "{}{}",
                                    SUMMARY_PREFIX, summary
                                )),
                            },
                        ];
                        debug!(
                            "!aip - reset history with summary, new length: {}",
                            hist.len()
                        );
                    }
                    hist.push(user_message);
                    debug!(
                        "!aip - added user message, history length now: {}",
                        hist.len()
                    );
                    Self::execute_chat_and_reply(&context, event, &mut hist).await?;
                    debug!(
                        "!aip - after AI response, final history length: {}",
                        hist.len()
                    );
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl ChatApp {
    pub fn new(config: &'static Config) -> Self {
        Self {
            client: Client::new(),
            config,
            current_model: Arc::new(RwLock::new(config.default_model.clone())),
            history: Arc::new(DashMap::new()),
            rate_limiter: Arc::new(DashMap::new()),
        }
    }

    fn create_system_message(config: &Config) -> ChatMessage {
        ChatMessage {
            role: "system".to_string(),
            content: ChatContent::Text(config.system_prompt.clone()),
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
                Segment::Image {
                    file,
                    url: Some(url),
                    ..
                } => {
                    images_to_process.push((file.clone(), url.clone()));
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
        match file_name.split('.').next_back() {
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("webp") => "image/webp",
            _ => "application/octet-stream",
        }
    }

    async fn check_rate_limit(
        rate_limiter: &Arc<DashMap<i64, Vec<Instant>>>,
        user_id: i64,
        config: &Config,
    ) -> Result<(), String> {
        let now = Instant::now();
        let mut requests = rate_limiter.entry(user_id).or_default();
        requests.retain(|&t| now.duration_since(t).as_secs() < config.rate_limit_window_secs);

        debug!(
            "Rate limit check - user_id: {}, current requests in window: {}",
            user_id,
            requests.len()
        );

        if requests.len() >= config.rate_limit_max {
            debug!("Rate limit exceeded for user_id: {}", user_id);
            return Err(RATE_LIMIT_MESSAGE.to_string());
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
        debug!(
            "execute_chat_and_reply - input messages count: {}",
            messages.len()
        );
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
        debug!(
            "execute_chat_and_reply - after adding AI response, messages count: {}",
            messages.len()
        );

        Self::send_reply(context.config, event, res_text).await
    }

    pub fn split_text_by_length(text: &str, max_length: std::num::NonZeroUsize) -> Vec<String> {
        let max_length = max_length.get();

        let mut result = Vec::new();
        let mut current_start = 0;
        let mut last_char_end = 0;

        for (i, ch) in text.char_indices() {
            let char_end = i + ch.len_utf8();
            if char_end - current_start > max_length {
                result.push(text[current_start..last_char_end].to_string());
                current_start = last_char_end;
            }
            last_char_end = char_end;
        }

        if current_start < text.len() {
            result.push(text[current_start..].to_string());
        }

        result
    }

    async fn send_reply(config: &Config, event: &MessageEvent, text: String) -> Result<()> {
        // First, split the text into chunks if it's too long
        let text_chunks = Self::split_text_by_length(&text, config.max_message_length);

        let url_re = Regex::new(r"(https?://[\S]+\.(?:png|jpg|jpeg|gif|webp))").unwrap();
        for (index, chunk) in text_chunks.iter().enumerate() {
            // For each chunk, process it for images and send
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
                tokio::time::sleep(tokio::time::Duration::from_millis(REPLY_DELAY_MS)).await;
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
            .post(format!("{}/v1/chat/completions", context.config.base_url))
            .bearer_auth(&context.config.token)
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
            content: ChatContent::Text(SUMMARY_PROMPT.to_string()),
        }];
        summary_messages.extend_from_slice(messages);

        let res = Self::call_api(context, &summary_messages).await?;
        match res.content {
            ChatContent::Text(text) => Ok(text),
            _ => Err(anyhow!("Unsupported summary response from API")),
        }
    }
}
