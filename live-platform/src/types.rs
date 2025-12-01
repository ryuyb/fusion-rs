use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Platform {
    Douyu,
    Bilibili,
}

impl Display for Platform {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for Platform {
    type Err = ();

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string.to_ascii_lowercase().as_str() {
            "douyu" => Ok(Platform::Douyu),
            "bilibili" => Ok(Platform::Bilibili),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamerInfo {
    pub platform: Platform,
    pub platform_streamer_id: String,
    pub name: String,
    pub avatar: String,
    pub description: String,
    pub room_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LiveStatus {
    pub is_live: bool,
    pub title: String,
    pub game_name: String,
    pub start_time: Option<NaiveDateTime>,
    pub viewer_count: u64,
    pub cover_image: String,
}

#[async_trait]
pub trait LivePlatform: Send + Sync {
    fn platform(&self) -> Platform;

    async fn fetch_streamer_info(&self, platform_streamer_id: String) -> Result<StreamerInfo>;

    async fn check_live_status(&self, platform_streamer_id: String) -> Result<LiveStatus>;
}
