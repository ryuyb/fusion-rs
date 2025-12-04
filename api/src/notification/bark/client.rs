use anyhow::Context;
use migration::async_trait::async_trait;

use crate::notification::bark::msg::{BarkResponse, Msg};
use crate::notification::provider::{
    NotificationAddress, NotificationChannel, NotificationMessage, NotificationProvider,
};

const BASE_URL: &str = "https://api.day.app";

pub struct BarkProvider {
    client: reqwest::Client,
}

impl BarkProvider {
    pub fn new() -> anyhow::Result<Self> {
        let client = reqwest::ClientBuilder::default()
            .build()
            .context("failed to build reqwest client")?;
        Ok(Self { client })
    }

    async fn push(&self, device_key: &str, msg: &Msg) -> anyhow::Result<()> {
        let response = self
            .client
            .post(format!("{}/{}", BASE_URL, device_key))
            .header("Content-Type", "application/json")
            .json(msg)
            .send()
            .await
            .context("bark: failed to send message to device")?
            .json::<BarkResponse>()
            .await?;

        if response.code != 200 {
            anyhow::bail!(
                "failed to send message to device: {}, message: {}",
                response.code,
                response.message
            );
        }

        Ok(())
    }

    fn build_message(notification: &NotificationMessage, address: &NotificationAddress) -> Msg {
        let mut msg = Msg::new(notification.title(), notification.body());

        if let Some(url) = notification.url() {
            msg.set_url(url);
        }

        if let Some(sound) = notification
            .metadata_value("sound")
            .or_else(|| address.metadata_value("sound"))
        {
            msg.set_sound(sound);
        }

        if let Some(group) = address
            .metadata_value("group")
            .or_else(|| notification.metadata_value("group"))
        {
            msg.set_group(group);
        }

        msg
    }
}

#[async_trait]
impl NotificationProvider for BarkProvider {
    fn channel(&self) -> NotificationChannel {
        NotificationChannel::Bark
    }

    async fn send(
        &self,
        address: &NotificationAddress,
        message: &NotificationMessage,
    ) -> anyhow::Result<()> {
        let msg = Self::build_message(message, address);
        self.push(address.destination(), &msg).await
    }
}
