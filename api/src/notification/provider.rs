use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use migration::async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotificationChannel {
    Bark,
}

#[derive(Debug, Clone, Default)]
pub struct NotificationMessage {
    title: String,
    body: String,
    url: Option<String>,
    metadata: HashMap<String, String>,
}

impl NotificationMessage {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            url: None,
            metadata: HashMap::new(),
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn metadata_value(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|value| value.as_str())
    }

    pub fn set_url(&mut self, url: impl Into<String>) -> &mut Self {
        self.url = Some(url.into());
        self
    }

    pub fn insert_metadata(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> &mut Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct NotificationAddress {
    channel: NotificationChannel,
    destination: String,
    metadata: HashMap<String, String>,
}

impl NotificationAddress {
    pub fn new(channel: NotificationChannel, destination: impl Into<String>) -> Self {
        Self {
            channel,
            destination: destination.into(),
            metadata: HashMap::new(),
        }
    }

    pub fn channel(&self) -> NotificationChannel {
        self.channel
    }

    pub fn destination(&self) -> &str {
        &self.destination
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn metadata_value(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|value| value.as_str())
    }

    pub fn insert_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct NotificationTarget {
    user_id: Uuid,
    addresses: Vec<NotificationAddress>,
}

impl NotificationTarget {
    pub fn new(user_id: Uuid, addresses: Vec<NotificationAddress>) -> Self {
        Self { user_id, addresses }
    }

    pub fn single(user_id: Uuid, address: NotificationAddress) -> Self {
        Self {
            user_id,
            addresses: vec![address],
        }
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn addresses(&self) -> &[NotificationAddress] {
        &self.addresses
    }
}

#[async_trait]
pub trait NotificationProvider: Send + Sync + 'static {
    fn channel(&self) -> NotificationChannel;

    async fn send(
        &self,
        address: &NotificationAddress,
        message: &NotificationMessage,
    ) -> Result<()>;
}

#[derive(Default)]
pub struct NotificationCenter {
    providers: HashMap<NotificationChannel, Arc<dyn NotificationProvider>>,
}

impl NotificationCenter {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn with_providers<I>(providers: I) -> Self
    where
        I: IntoIterator<Item = Arc<dyn NotificationProvider>>,
    {
        let mut center = Self::new();
        for provider in providers {
            center.register(provider);
        }
        center
    }

    pub fn register(&mut self, provider: Arc<dyn NotificationProvider>) -> &mut Self {
        self.providers.insert(provider.channel(), provider);
        self
    }

    pub async fn notify_target(
        &self,
        target: &NotificationTarget,
        message: &NotificationMessage,
    ) -> Result<()> {
        for address in target.addresses() {
            self.notify_address(address, message)
                .await
                .with_context(|| format!("failed to notify user {}", target.user_id()))?;
        }
        Ok(())
    }

    pub async fn notify_address(
        &self,
        address: &NotificationAddress,
        message: &NotificationMessage,
    ) -> Result<()> {
        let provider = self
            .providers
            .get(&address.channel())
            .ok_or_else(|| anyhow!("no provider configured for channel {:?}", address.channel()))?;

        provider
            .send(address, message)
            .await
            .with_context(|| format!("channel {:?}", address.channel()))
    }
}
