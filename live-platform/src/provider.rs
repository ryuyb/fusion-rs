use crate::bilibili::Bilibili;
use crate::douyu::Douyu;
use crate::types::{LivePlatform, Platform};
use crate::{LiveStatus, StreamerInfo};
use anyhow::{Result, anyhow};
use std::collections::HashMap;

pub struct LivePlatformProvider {
    platforms: HashMap<Platform, Box<dyn LivePlatform>>,
}

impl LivePlatformProvider {
    pub fn new() -> Result<Self> {
        let mut platforms: HashMap<Platform, Box<dyn LivePlatform>> = HashMap::new();

        let bilibili = Bilibili::new()?;
        platforms.insert(Platform::Bilibili, Box::new(bilibili));

        let douyu = Douyu::new()?;
        platforms.insert(Platform::Douyu, Box::new(douyu));

        Ok(Self { platforms })
    }

    pub fn get_platform(&self, platform: &Platform) -> Option<&Box<dyn LivePlatform>> {
        self.platforms.get(platform)
    }

    pub async fn fetch_streamer_info(
        &self,
        platform: Platform,
        room_id: String,
    ) -> Result<StreamerInfo> {
        let p = self
            .get_platform(&platform)
            .ok_or(anyhow!("Unsupported platform: {}", platform))?;
        p.fetch_streamer_info(room_id).await
    }

    pub async fn check_live_status(
        &self,
        platform: Platform,
        room_id: String,
    ) -> Result<LiveStatus> {
        let p = self
            .get_platform(&platform)
            .ok_or(anyhow!("Unsupported platform: {}", platform))?;
        p.check_live_status(room_id).await
    }
}
