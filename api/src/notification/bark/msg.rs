use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Push Notification Message structure.
///
/// This struct represents a push notification message that can be sent to devices.
/// It contains various fields to customize the notification's behavior and appearance.
///
/// # Example
/// ```rust
/// use bark::msg::Msg;
///
/// // new a simple message with title and body
/// let msg = Msg::new("title", "body");
///
/// // new a message with title = Notification and body
/// let mut msg = Msg::with_body("body");
///
/// // set some fields
/// msg.set_level(Level::Active);
/// msg.set_badge(1);
/// // and so on
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct Msg {
    /// Push Title
    title: String,

    /// Push content
    body: String,

    /// Push Interruption Level
    ///
    /// active: Default value, the system will immediately display the notification on the screen.
    ///
    /// timeSensitive: Time-sensitive notification, can be displayed while in focus mode.
    ///
    /// passive: Only adds the notification to the notification list, will not display on the screen.
    #[serde(skip_serializing_if = "Option::is_none")]
    level: Option<Level>,

    /// Push Badge, can be any number
    #[serde(skip_serializing_if = "Option::is_none")]
    badge: Option<u64>,

    /// Pass 0 to disable; Automatically copy push content below iOS 14.5; above iOS 14.5, you need to manually long-press the push or pull down the push
    #[serde(skip_serializing_if = "Option::is_none")]
    auto_copy: Option<u8>,

    /// When copying the push, specify the content to copy; if this parameter is not provided, the entire push content will be copied
    #[serde(skip_serializing_if = "Option::is_none")]
    copy: Option<String>,

    /// You can set different ringtones for the push
    #[serde(skip_serializing_if = "Option::is_none")]
    sound: Option<String>,

    /// Set a custom icon for the push; the set icon will replace the default Bark icon
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,

    /// Group messages; pushes will be displayed in groups in the notification center
    #[serde(skip_serializing_if = "Option::is_none")]
    group: Option<String>,

    /// Pass 1 to save the push; passing anything else will not save the push; if not passed, it will be decided according to the app's internal settings
    #[serde(skip_serializing_if = "Option::is_none")]
    is_archive: Option<u8>,

    /// The URL to jump to when clicking the push, supports URL Scheme and Universal Link
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,

    /// message id
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,

    /// delete flag
    #[serde(skip_serializing_if = "Option::is_none")]
    is_deleted: Option<bool>,
}

/// Notification level
///
/// active: Default value, the system will immediately display the notification on the screen.
///
/// timeSensitive: Time-sensitive notification, can be displayed while in focus mode.
///
/// passive: Only adds the notification to the notification list, will not display on the screen.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Level {
    Active,
    TimeSensitive,
    Passive,
}

#[allow(dead_code)] // Public builder helpers retained for potential external consumers
impl Level {
    pub fn from_str(str: &str) -> Option<Self> {
        match str.to_lowercase().as_str() {
            "timesensitive" => Some(Self::TimeSensitive),
            "passive" => Some(Self::Passive),
            "active" => Some(Self::Active),
            _ => None,
        }
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Level::Active => "active",
            Level::TimeSensitive => "timeSensitive",
            Level::Passive => "passive",
        };
        write!(f, "{}", str)
    }
}

#[allow(dead_code)] // Rich builder surface kept for callers beyond current crate usage
impl Msg {
    /// Creates a new `Msg` instance with a title and body.
    ///
    /// # Arguments
    /// - `title`: The title of the notification.
    /// - `body`: The content/body of the notification.
    ///
    /// # Returns
    /// A new `Msg` instance.
    pub fn new(title: &str, body: &str) -> Self {
        Msg {
            ..Self::default(Some(title.to_string()), body.to_string())
        }
    }

    /// Creates a new `Msg` instance with only a body.
    ///
    /// # Arguments
    /// - `body`: The content/body of the notification.
    ///
    /// # Returns
    /// A new `Msg` instance with the title set to "Notification".
    pub fn with_body(body: &str) -> Self {
        Msg {
            ..Self::default(None, body.to_string())
        }
    }

    /// Creates a default `Msg` instance.
    ///
    /// # Arguments
    /// - `title`: An optional title for the notification.
    /// - `body`: The content/body of the notification.
    ///
    /// # Returns
    /// A new `Msg` instance with default values.
    fn default(title: Option<String>, body: String) -> Self {
        Msg {
            title: title.unwrap_or("Notification".to_string()),
            body,
            level: None,
            badge: None,
            auto_copy: None,
            copy: None,
            sound: Some("chime.caf".to_string()),
            icon: Some("https://github.com/66f94eae/bark-dev/raw/main/bot.jpg".to_string()),
            group: None,
            is_archive: None,
            url: None,
            id: None,
            is_deleted: None,
        }
    }

    pub fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    pub fn is_deleted(&self) -> bool {
        self.is_deleted.unwrap_or(false)
    }

    /// Sets the interruption level of the notification.
    ///
    /// # Arguments
    /// - `level`: The interruption level [`Level`]
    ///
    /// # Returns
    /// A mutable reference to `self` for method chaining.
    pub fn set_level(&mut self, level: Level) -> &mut Self {
        self.level = Some(level);
        self
    }

    /// Sets the badge number.
    ///
    /// # Arguments
    /// - `badge`: The badge number to display on the app icon.
    ///
    /// # Returns
    /// A mutable reference to `self` for method chaining.
    pub fn set_badge(&mut self, badge: u64) -> &mut Self {
        if badge > 0 {
            self.badge = Some(badge);
        } else {
            self.badge = None;
        }
        self
    }

    /// Sets whether to automatically copy the notification content.
    ///
    /// # Arguments
    /// - `auto_copy`: false to disable, true to enable.
    ///
    /// # Returns
    /// A mutable reference to `self` for method chaining.
    pub fn set_auto_copy(&mut self, auto_copy: bool) -> &mut Self {
        match auto_copy {
            false => self.auto_copy = Some(0),
            true => self.auto_copy = None,
        }
        self
    }

    /// Sets specific content to copy when the notification is copied.
    ///
    /// # Arguments
    /// - `copy`: The content to copy.
    ///
    /// # Returns
    /// A mutable reference to `self` for method chaining.
    pub fn set_copy(&mut self, copy: &str) -> &mut Self {
        if copy.trim().is_empty() {
            self.copy = None;
        } else {
            self.copy = Some(copy.to_string());
        }
        self
    }

    /// Sets the sound file to play with the notification.
    ///
    /// # Arguments
    /// - `sound`: The sound file name.
    ///
    /// # Returns
    /// A mutable reference to `self` for method chaining.
    pub fn set_sound(&mut self, sound: &str) -> &mut Self {
        self.sound = Some(sound.to_string());
        self
    }

    /// Sets a custom icon URL for the notification.
    ///
    /// # Arguments
    /// - `icon`: The icon URL.
    ///
    /// # Returns
    /// A mutable reference to `self` for method chaining.
    pub fn set_icon(&mut self, icon: &str) -> &mut Self {
        if icon.trim().is_empty() {
            self.icon = None;
        } else {
            self.icon = Some(icon.to_string());
        }
        self
    }

    /// Sets the group identifier for notifications.
    ///
    /// # Arguments
    /// - `group`: The group identifier.
    ///
    /// # Returns
    /// A mutable reference to `self` for method chaining.
    pub fn set_group(&mut self, group: &str) -> &mut Self {
        self.group = Some(group.to_string());
        self
    }

    /// Sets whether to archive the notification.
    ///
    /// # Arguments
    /// - `is_archive`: true to save, false to not save.
    ///
    /// # Returns
    /// A mutable reference to `self` for method chaining.
    pub fn set_is_archive(&mut self, is_archive: bool) -> &mut Self {
        match is_archive {
            true => self.is_archive = Some(1),
            false => self.is_archive = None,
        }
        self
    }

    /// Sets the URL to open when the notification is clicked.
    ///
    /// # Arguments
    /// - `url`: The URL.
    ///
    /// # Returns
    /// A mutable reference to `self` for method chaining.
    pub fn set_url(&mut self, url: &str) -> &mut Self {
        if url.trim().is_empty() {
            self.url = None;
        } else {
            self.url = Some(url.to_string());
        }
        self
    }

    pub fn set_id(&mut self, msg_id: &str) -> &mut Self {
        if msg_id.len() >= 64 {
            panic!("Invalid msg_id length.The value of this key must not exceed 64 bytes.");
        }
        self.id = Some(msg_id.to_string());
        self
    }

    pub fn set_deleted(&mut self) -> &mut Self {
        self.is_deleted = Some(true);
        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BarkResponse {
    pub code: i64,
    pub message: String,
}
