use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;

use crate::protocol::event::{Event, MessageEvent};
use crate::protocol::message::Segment;

const API_URL: &str = "https://api.thecatapi.com/v1/images/search";
const MAX_IMAGES: usize = 3;

#[derive(Deserialize)]
struct CatApiResponse {
    url: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub white_groups: Vec<i64>,
}

pub struct CatApp {
    config: &'static Config,
}

#[async_trait]
impl super::Application for CatApp {
    fn name(&self) -> &str {
        "cat"
    }

    async fn on_event(&mut self, event: Arc<Event>) -> Result<()> {
        if let Event::MessageEvent(msg_event) = event.as_ref() {
            self.handle_message_event(msg_event).await?;
        }
        Ok(())
    }
}

impl CatApp {
    pub fn new(config: &'static Config) -> Self {
        Self { config }
    }

    async fn handle_message_event(&self, event: &MessageEvent) -> Result<()> {
        if let MessageEvent::Group(event) = event
            && !self.config.white_groups.contains(&event.group_id)
        {
            // white list mode
            return Ok(());
        }

        let raw_message = event.raw_message();
        let count = raw_message.matches('喵').count();

        if count > 0 {
            let limit = count.min(MAX_IMAGES);
            let img_urls = self.fetch_cat_images(limit).await?;

            // Take only the number of images requested (up to count)
            let images_to_send: Vec<Segment> = img_urls
                .into_iter()
                .take(count)
                .map(Segment::image)
                .collect();

            if !images_to_send.is_empty() {
                let num_images = images_to_send.len();
                event.reply(images_to_send, false).await?;
                log::debug!("Sent {} cat images to user {}", num_images, event.user_id());
            }
        }

        Ok(())
    }

    async fn fetch_cat_images(&self, limit: usize) -> Result<Vec<String>> {
        let client = reqwest::Client::new();
        let response = client
            .get(API_URL)
            .query(&[("limit", limit)])
            .send()
            .await?;

        log::debug!("Cat API request with limit: {}", limit);

        let cat_responses: Vec<CatApiResponse> = response.json().await?;
        log::debug!("Received {} cat images from API", cat_responses.len());

        let img_urls: Vec<String> = cat_responses.into_iter().map(|cat| cat.url).collect();

        Ok(img_urls)
    }
}
