use crate::bilibili::Bilibili;
use crate::douyu::Douyu;
use crate::types::{LivePlatform, Platform};
use crate::{LiveStatus, StreamerInfo};
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;

pub struct LivePlatformProvider {
    platforms: HashMap<Platform, Arc<dyn LivePlatform>>,
}

impl LivePlatformProvider {
    pub fn new() -> Result<Self> {
        let mut provider = Self {
            platforms: HashMap::new(),
        };

        provider.register(Bilibili::new()?);
        provider.register(Douyu::new()?);

        Ok(provider)
    }

    pub fn register<P>(&mut self, platform: P) -> &mut Self
    where
        P: LivePlatform + 'static,
    {
        self.register_arc(Arc::new(platform))
    }

    pub fn register_arc(&mut self, provider: Arc<dyn LivePlatform>) -> &mut Self {
        let platform = provider.platform();
        self.platforms.insert(platform, provider);
        self
    }

    pub fn get(&self, platform: Platform) -> Option<&dyn LivePlatform> {
        self.platforms
            .get(&platform)
            .map(|provider| provider.as_ref())
    }

    fn provider(&self, platform: Platform) -> Result<&dyn LivePlatform> {
        self.get(platform)
            .ok_or_else(|| anyhow!("Unsupported platform: {}", platform))
    }

    pub async fn fetch_streamer_info(
        &self,
        platform: Platform,
        room_id: impl AsRef<str>,
    ) -> Result<StreamerInfo> {
        let provider = self.provider(platform)?;
        provider.fetch_streamer_info(room_id.as_ref()).await
    }

    pub async fn check_live_status(
        &self,
        platform: Platform,
        room_id: impl AsRef<str>,
    ) -> Result<LiveStatus> {
        let provider = self.provider(platform)?;
        provider.check_live_status(room_id.as_ref()).await
    }
}
