//! Configuration Module
//!
//! Manages persistent configuration for the messaging popup including
//! enabled messengers, popup settings, and notification preferences.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, error, info};

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Which messengers to handle
    pub enabled_messengers: Vec<MessengerConfig>,

    /// Popup window settings
    pub popup: PopupConfig,

    /// Notification settings
    pub notifications: NotificationConfig,

    /// Whether configuration has unsaved changes (not serialized)
    #[serde(skip)]
    pub dirty: bool,
}

/// Configuration for individual messenger services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessengerConfig {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Android package name for notification matching
    pub package_name: String,
    /// Web interface URL
    pub web_url: String,
    /// Optional custom icon name
    pub icon: Option<String>,
    /// Whether this messenger is enabled
    pub enabled: bool,
}

/// Popup window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopupConfig {
    /// Width of popup window
    pub width: u32,
    /// Height of popup window
    pub height: u32,
    /// Position: "cursor", "center", "bottom-right"
    pub position: PopupPosition,
    /// Keep popup open when clicking outside
    pub persistent: bool,
    /// Remember last used messenger
    pub remember_last: bool,
    /// Last used messenger id
    pub last_messenger: Option<String>,
}

/// Popup positioning options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PopupPosition {
    /// Position near the cursor
    Cursor,
    /// Center of the screen
    Center,
    /// Bottom right corner
    #[default]
    BottomRight,
}

impl PopupPosition {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cursor => "cursor",
            Self::Center => "center",
            Self::BottomRight => "bottom-right",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "cursor" => Self::Cursor,
            "center" => Self::Center,
            "bottom-right" | "bottomright" => Self::BottomRight,
            _ => Self::BottomRight,
        }
    }

    pub fn all() -> &'static [PopupPosition] {
        &[Self::Cursor, Self::Center, Self::BottomRight]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Cursor => "Near Cursor",
            Self::Center => "Center",
            Self::BottomRight => "Bottom Right",
        }
    }
}

/// Notification behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Show desktop notifications
    pub show_notifications: bool,
    /// Play notification sound
    pub play_sound: bool,
    /// Auto-open popup on notification
    pub auto_open: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled_messengers: vec![
                MessengerConfig {
                    id: "google-messages".to_string(),
                    name: "Google Messages".to_string(),
                    package_name: "com.google.android.apps.messaging".to_string(),
                    web_url: "https://messages.google.com/web".to_string(),
                    icon: Some("google-messages".to_string()),
                    enabled: true,
                },
                MessengerConfig {
                    id: "whatsapp".to_string(),
                    name: "WhatsApp".to_string(),
                    package_name: "com.whatsapp".to_string(),
                    web_url: "https://web.whatsapp.com".to_string(),
                    icon: Some("whatsapp".to_string()),
                    enabled: true,
                },
                MessengerConfig {
                    id: "telegram".to_string(),
                    name: "Telegram".to_string(),
                    package_name: "org.telegram.messenger".to_string(),
                    web_url: "https://web.telegram.org".to_string(),
                    icon: Some("telegram".to_string()),
                    enabled: true,
                },
                MessengerConfig {
                    id: "signal".to_string(),
                    name: "Signal".to_string(),
                    package_name: "org.thoughtcrime.securesms".to_string(),
                    // Signal has no web client - link to desktop app download
                    web_url: "https://signal.org/download/".to_string(),
                    icon: Some("signal".to_string()),
                    enabled: false,
                },
                MessengerConfig {
                    id: "discord".to_string(),
                    name: "Discord".to_string(),
                    package_name: "com.discord".to_string(),
                    web_url: "https://discord.com/app".to_string(),
                    icon: Some("discord".to_string()),
                    enabled: false,
                },
                MessengerConfig {
                    id: "slack".to_string(),
                    name: "Slack".to_string(),
                    package_name: "com.Slack".to_string(),
                    web_url: "https://app.slack.com".to_string(),
                    icon: Some("slack".to_string()),
                    enabled: false,
                },
            ],
            popup: PopupConfig {
                width: 420,
                height: 650,
                position: PopupPosition::BottomRight,
                persistent: false,
                remember_last: true,
                last_messenger: None,
            },
            notifications: NotificationConfig {
                show_notifications: true,
                play_sound: true,
                auto_open: false,
            },
            dirty: false,
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load() -> Option<Self> {
        let path = Self::config_path()?;
        debug!("Loading config from {:?}", path);

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                debug!("Could not read config file: {}", e);
                return None;
            }
        };

        match ron::from_str(&content) {
            Ok(config) => {
                info!("Loaded configuration from {:?}", path);
                Some(config)
            }
            Err(e) => {
                error!("Failed to parse config: {}", e);
                None
            }
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path().ok_or_else(|| anyhow::anyhow!("No config path"))?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())?;
        std::fs::write(&path, content)?;
        info!("Saved configuration to {:?}", path);
        Ok(())
    }

    /// Get the configuration file path
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("cosmic").join("org.cosmicde.MessagesPopup.ron"))
    }

    /// Get the data directory for WebView storage
    pub fn data_dir() -> Option<PathBuf> {
        dirs::data_local_dir().map(|dir| dir.join("cosmic-messages-popup"))
    }

    /// Check if a messenger is enabled
    pub fn is_messenger_enabled(&self, id: &str) -> bool {
        self.enabled_messengers
            .iter()
            .any(|m| m.id == id && m.enabled)
    }

    /// Toggle a messenger's enabled state
    pub fn toggle_messenger(&mut self, id: &str, enabled: bool) {
        if let Some(messenger) = self.enabled_messengers.iter_mut().find(|m| m.id == id) {
            messenger.enabled = enabled;
            self.mark_dirty();
        }
    }

    /// Find messenger config by package name
    pub fn find_messenger_by_package(&self, package_name: &str) -> Option<&MessengerConfig> {
        self.enabled_messengers
            .iter()
            .find(|m| m.package_name == package_name && m.enabled)
    }

    /// Get all enabled messengers
    pub fn get_enabled_messengers(&self) -> Vec<&MessengerConfig> {
        self.enabled_messengers
            .iter()
            .filter(|m| m.enabled)
            .collect()
    }

    /// Add a custom messenger
    pub fn add_custom_messenger(&mut self, id: String, name: String, url: String) {
        self.enabled_messengers.push(MessengerConfig {
            id,
            name,
            package_name: String::new(),
            web_url: url,
            icon: None,
            enabled: true,
        });
        self.mark_dirty();
    }

    /// Mark the configuration as having unsaved changes
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Save configuration if it has unsaved changes
    pub fn save_if_dirty(&mut self) -> anyhow::Result<()> {
        if self.dirty {
            self.save()?;
            self.dirty = false;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(!config.enabled_messengers.is_empty());
        assert!(config.is_messenger_enabled("google-messages"));
        assert!(config.is_messenger_enabled("whatsapp"));
        assert!(!config.is_messenger_enabled("discord"));
    }

    #[test]
    fn test_toggle_messenger() {
        let mut config = Config::default();
        assert!(config.is_messenger_enabled("google-messages"));

        config.toggle_messenger("google-messages", false);
        assert!(!config.is_messenger_enabled("google-messages"));

        config.toggle_messenger("google-messages", true);
        assert!(config.is_messenger_enabled("google-messages"));
    }

    #[test]
    fn test_find_by_package() {
        let config = Config::default();

        let messenger = config.find_messenger_by_package("com.google.android.apps.messaging");
        assert!(messenger.is_some());
        assert_eq!(messenger.unwrap().id, "google-messages");

        let not_found = config.find_messenger_by_package("com.unknown.app");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_popup_position() {
        assert_eq!(PopupPosition::from_str("cursor"), PopupPosition::Cursor);
        assert_eq!(PopupPosition::from_str("center"), PopupPosition::Center);
        assert_eq!(
            PopupPosition::from_str("bottom-right"),
            PopupPosition::BottomRight
        );
        assert_eq!(
            PopupPosition::from_str("unknown"),
            PopupPosition::BottomRight
        );
    }

    #[test]
    fn test_serialization() {
        let config = Config::default();
        let serialized = ron::ser::to_string_pretty(&config, ron::ser::PrettyConfig::default());
        assert!(serialized.is_ok());

        let deserialized: Result<Config, _> = ron::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_dirty_flag() {
        let mut config = Config::default();
        assert!(!config.dirty);

        config.mark_dirty();
        assert!(config.dirty);

        config.toggle_messenger("google-messages", false);
        assert!(config.dirty);
    }

    #[test]
    fn test_dirty_flag_not_serialized() {
        let mut config = Config::default();
        config.mark_dirty();
        assert!(config.dirty);

        let serialized = ron::ser::to_string_pretty(&config, ron::ser::PrettyConfig::default())
            .expect("Failed to serialize");
        let deserialized: Config =
            ron::from_str(&serialized).expect("Failed to deserialize");

        // dirty flag should be false after deserialization (not saved)
        assert!(!deserialized.dirty);
    }

    #[test]
    fn test_signal_web_url_updated() {
        let config = Config::default();
        let signal = config
            .enabled_messengers
            .iter()
            .find(|m| m.id == "signal")
            .expect("Signal config not found");
        assert_eq!(signal.web_url, "https://signal.org/download/");
    }
}
