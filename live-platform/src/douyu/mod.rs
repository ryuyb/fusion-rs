mod dto;

use crate::douyu::dto::BetardResponse;
use crate::{LivePlatform, LiveStatus, Platform, StreamerInfo};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use chrono::DateTime;
use reqwest::header::{HeaderValue, CONTENT_TYPE};
use scraper::{Html, Selector};

const BASE_URL: &str = "https://douyu.com";

pub struct Douyu {
    client: reqwest::Client,
}

impl Douyu {
    pub fn new() -> Result<Self> {
        let client = reqwest::ClientBuilder::default()
            .build()
            .context("Failed to build reqwest client")?;
        Ok(Self { client })
    }

    async fn fetch_betard_info(&self, platform_streamer_id: String) -> Result<BetardResponse> {
        let response = self
            .client
            .get(format!(
                "https://www.douyu.com/betard/{platform_streamer_id}"
            ))
            .send()
            .await
            .context("Failed to query douyu betard info")?;

        let content_type = response.headers().get(CONTENT_TYPE).cloned();

        let resp_text = response
            .text()
            .await
            .context("Failed to parse douyu betard info response")?;

        if Self::betard_is_prompt_html(content_type, &resp_text) {
            return if let Some(message) = Self::betard_extract_prompt_messgae(&resp_text) {
                Err(anyhow!("Douyu betard error with message: {}", message))
            } else {
                Err(anyhow::anyhow!(
                    "Douyu betard is prompt but did not receive message"
                ))
            }
        }

        let betard = serde_json::from_str::<BetardResponse>(&resp_text)
            .context("Failed to parse douyu betard info from json")?;
        Ok(betard)
    }

    fn betard_is_prompt_html(content_type: Option<HeaderValue>, text: &str) -> bool {
        if let Some(ct) = content_type {
            if let Ok(ct_str) = ct.to_str() {
                return ct_str.to_ascii_lowercase().contains("text/html");
            }
        }
        text.contains("<title>提示信息 -斗鱼</title>")
    }

    fn betard_extract_prompt_messgae(text: &str) -> Option<String> {
        let document = Html::parse_document(text);
        let selector = match Selector::parse(".error > span > p") {
            Ok(selector) => selector,
            Err(_) => {
                log::error!("Failed parse selector `.error > span > p`");
                return None;
            }
        };
        if let Some(elem) = document.select(&selector).next() {
            return Some(elem.text().collect::<String>().trim().to_string());
        }
        None
    }
}

#[async_trait]
impl LivePlatform for Douyu {
    fn platform(&self) -> Platform {
        Platform::Douyu
    }

    async fn fetch_streamer_info(&self, platform_streamer_id: String) -> Result<StreamerInfo> {
        let response = self.fetch_betard_info(platform_streamer_id.clone()).await?;

        Ok(StreamerInfo {
            platform: Platform::Douyu,
            platform_streamer_id: platform_streamer_id.clone(),
            name: response.room.room_name,
            avatar: response.room.avatar.big,
            description: response.room.show_details,
            room_url: format!("{BASE_URL}/{platform_streamer_id}"),
        })
    }

    async fn check_live_status(&self, platform_streamer_id: String) -> Result<LiveStatus> {
        let response = self.fetch_betard_info(platform_streamer_id).await?;

        let start_time =
            DateTime::from_timestamp(response.room.show_time, 0).map(|t| t.naive_utc());

        Ok(LiveStatus {
            is_live: response.room.show_status == 1,
            title: response.room.room_name,
            game_name: response.room.second_lvl_name,
            start_time,
            viewer_count: response.room.room_biz_all.hot.parse::<u64>().unwrap_or(0),
            cover_image: response.room.cover_src,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn douyu() -> Douyu {
        Douyu::new().unwrap()
    }

    #[tokio::test]
    async fn test_fetch_streamer_info() {
        let result = douyu().fetch_streamer_info("60937".to_string()).await;
        match result {
            Ok(v) => println!("{:?}", v),
            Err(error) => println!("debug error: {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_fetch_live_status() {
        let result = douyu().check_live_status("60937".to_string()).await;
        match result {
            Ok(v) => println!("{:?}", v),
            Err(error) => println!("debug error: {:?}", error),
        }
    }
}
