use crate::notification::bark::msg::{BarkResponse, Msg};
use anyhow::Context;

const BASE_URL: &str = "https://api.day.app";

pub struct Bark {
    client: reqwest::Client,
}

impl Bark {
    pub fn new() -> anyhow::Result<Self> {
        let client = reqwest::ClientBuilder::default()
            .build()
            .context("failed to build reqwest client")?;
        Ok(Self { client })
    }

    pub async fn send(&self, device_key: String, msg: &Msg) -> anyhow::Result<()> {
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
}
