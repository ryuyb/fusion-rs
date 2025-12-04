mod dto;

use crate::bilibili::dto::{MasterInfo, MasterInfoResp, RespWrapper, RoomInfoResp};
use crate::types::{LivePlatform, LiveStatus, Platform, StreamerInfo};
use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use chrono::NaiveDateTime;
use log::error;

const BASE_URL: &str = "https://live.bilibili.com";

pub struct Bilibili {
    client: reqwest::Client,
}

impl Bilibili {
    pub fn new() -> Result<Self> {
        let client = reqwest::ClientBuilder::default()
            .build()
            .context("Failed to build reqwest client")?;
        Ok(Self { client })
    }

    async fn fetch_room_info(&self, platform_streamer_id: &str) -> Result<RoomInfoResp> {
        let resp = self
            .client
            .get("https://api.live.bilibili.com/room/v1/Room/get_info")
            .query(&[("room_id", platform_streamer_id)])
            .send()
            .await
            .context("Failed to query bilibili room info")?
            .json::<RespWrapper<RoomInfoResp>>()
            .await
            .context("Failed to parse room info response")?;
        if resp.code != 0 {
            error!(
                "Failed to query bilibili room info, resp code: {}, message: {}",
                resp.code, resp.message
            );
            return Err(anyhow!(
                "Failed to query bilibili room info, resp code: {}, message: {}",
                resp.code,
                resp.message
            ));
        }
        Ok(resp.data)
    }

    async fn fetch_master_info(&self, uid: i64) -> Result<MasterInfo> {
        let resp = self
            .client
            .get("https://api.live.bilibili.com/live_user/v1/Master/info")
            .query(&[("uid", uid)])
            .send()
            .await
            .context("Failed to query bilibili master info")?
            .json::<RespWrapper<MasterInfoResp>>()
            .await
            .context("Failed to parse master info response")?;
        if resp.code != 0 {
            error!(
                "Failed to query bilibili master info, resp code: {}, message: {}",
                resp.code, resp.message
            );
            return Err(anyhow!(
                "Failed to query bilibili master info, resp code: {}, message: {}",
                resp.code,
                resp.message
            ));
        }
        Ok(resp.data.info)
    }
}

#[async_trait]
impl LivePlatform for Bilibili {
    fn platform(&self) -> Platform {
        Platform::Bilibili
    }

    async fn fetch_streamer_info(&self, platform_streamer_id: &str) -> Result<StreamerInfo> {
        let room_info = self.fetch_room_info(platform_streamer_id).await?;
        let master_info = self.fetch_master_info(room_info.uid).await?;

        Ok(StreamerInfo {
            platform: Platform::Bilibili,
            platform_streamer_id: platform_streamer_id.to_string(),
            name: master_info.uname,
            avatar: master_info.face,
            description: room_info.description,
            room_url: format!("{BASE_URL}/{platform_streamer_id}"),
        })
    }

    async fn check_live_status(&self, platform_streamer_id: &str) -> Result<LiveStatus> {
        let resp = self.fetch_room_info(platform_streamer_id).await?;
        let mut start_time: Option<NaiveDateTime> = None;
        if resp.live_status == 1 && resp.live_time != "0000-00-00 00:00:00" {
            start_time =
                NaiveDateTime::parse_from_str(resp.live_time.as_str(), "%Y-%m-%d %H:%M:%S").ok();
        }
        Ok(LiveStatus {
            is_live: resp.live_status == 1,
            title: resp.title,
            game_name: resp.area_name,
            start_time,
            viewer_count: resp.online,
            cover_image: resp.user_cover,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_streamer_info() {
        let bilibili = Bilibili::new().unwrap();

        let result = bilibili.fetch_streamer_info("7734200").await;
        assert!(result.is_ok());
        println!("{:?}", result);
    }

    #[tokio::test]
    async fn test_check_live_status() {
        let bilibili = Bilibili::new().unwrap();

        let result = bilibili.check_live_status("7734200").await;
        assert!(result.is_ok());
        println!("{:?}", result);
    }
}
