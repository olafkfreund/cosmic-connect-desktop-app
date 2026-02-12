//! Notification Handling Module
//!
//! Displays COSMIC notifications when messages arrive and handles
//! the "Open" action to show the messaging popup.

use crate::config::Config;
use crate::dbus::NotificationData;
use notify_rust::Notification;
use tracing::{debug, error, info};

/// Messenger type derived from package name
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessengerType {
    GoogleMessages,
    WhatsApp,
    Telegram,
    Signal,
    Discord,
    Slack,
    Unknown,
}

#[allow(dead_code)]
impl MessengerType {
    /// Detect messenger type from Android package name
    pub fn from_package(package: &str) -> Self {
        match package {
            "com.google.android.apps.messaging" => Self::GoogleMessages,
            "com.whatsapp" | "com.whatsapp.w4b" => Self::WhatsApp,
            "org.telegram.messenger" | "org.telegram.messenger.web" => Self::Telegram,
            "org.thoughtcrime.securesms" => Self::Signal,
            "com.discord" => Self::Discord,
            "com.Slack" => Self::Slack,
            _ => Self::Unknown,
        }
    }

    /// Get the messenger ID used in configuration
    pub fn id(&self) -> &'static str {
        match self {
            Self::GoogleMessages => "google-messages",
            Self::WhatsApp => "whatsapp",
            Self::Telegram => "telegram",
            Self::Signal => "signal",
            Self::Discord => "discord",
            Self::Slack => "slack",
            Self::Unknown => "unknown",
        }
    }

    /// Get the icon name for this messenger
    pub fn icon(&self) -> &'static str {
        match self {
            Self::GoogleMessages => "google-messages-symbolic",
            Self::WhatsApp => "whatsapp-symbolic",
            Self::Telegram => "telegram-symbolic",
            Self::Signal => "signal-symbolic",
            Self::Discord => "discord-symbolic",
            Self::Slack => "slack-symbolic",
            Self::Unknown => "chat-symbolic",
        }
    }

    /// Get a fallback generic icon
    pub fn fallback_icon(&self) -> &'static str {
        match self {
            Self::GoogleMessages => "phone-symbolic",
            Self::WhatsApp | Self::Telegram | Self::Signal => "chat-symbolic",
            Self::Discord | Self::Slack => "system-users-symbolic",
            Self::Unknown => "mail-message-new-symbolic",
        }
    }

    /// Get the web URL for this messenger
    pub fn web_url(&self) -> &'static str {
        match self {
            Self::GoogleMessages => "https://messages.google.com/web",
            Self::WhatsApp => "https://web.whatsapp.com",
            Self::Telegram => "https://web.telegram.org",
            Self::Signal => "https://signal.org/download/",
            Self::Discord => "https://discord.com/app",
            Self::Slack => "https://app.slack.com",
            Self::Unknown => "",
        }
    }

    /// Get the keyboard shortcut index (1-based) for this messenger
    pub fn shortcut_index(&self) -> Option<u32> {
        match self {
            Self::GoogleMessages => Some(1),
            Self::WhatsApp => Some(2),
            Self::Telegram => Some(3),
            Self::Signal => Some(4),
            Self::Discord => Some(5),
            Self::Slack => Some(6),
            Self::Unknown => None,
        }
    }
}

/// Notification handler for displaying and managing message notifications
pub struct NotificationHandler {
    config: Config,
}

#[allow(dead_code)]
impl NotificationHandler {
    /// Create a new notification handler
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Update the configuration
    pub fn update_config(&mut self, config: Config) {
        self.config = config;
    }

    /// Handle an incoming notification
    ///
    /// Returns the messenger ID if a notification should be shown
    pub fn handle_notification(&self, data: &NotificationData) -> Option<String> {
        // Detect messenger type
        let messenger_type = MessengerType::from_package(&data.app_package);
        let messenger_id = messenger_type.id();

        debug!(
            "Handling notification from {} ({})",
            data.app_name, messenger_id
        );

        // Check if this messenger is enabled
        if !self.config.is_messenger_enabled(messenger_id) {
            debug!(
                "Messenger {} is disabled, ignoring notification",
                messenger_id
            );
            return None;
        }

        // Check if notifications are enabled globally
        if !self.config.notifications.show_notifications {
            debug!("Notifications are disabled globally");
            return None;
        }

        info!(
            "Processing notification: {} from {} ({})",
            data.title, data.app_name, messenger_id
        );

        // Display desktop notification
        let icon = messenger_type.icon();
        let summary = self.format_summary(data);
        let body = self.format_body(data);

        if let Err(e) = Notification::new()
            .summary(&summary)
            .body(&body)
            .icon(icon)
            .timeout(5000)
            .action("open", "Open")
            .action("dismiss", "Dismiss")
            .show()
        {
            error!("Failed to show notification: {}", e);
        }

        Some(messenger_id.to_string())
    }

    /// Check if auto-open is enabled
    pub fn should_auto_open(&self) -> bool {
        self.config.notifications.auto_open
    }

    /// Check if sound should be played
    pub fn should_play_sound(&self) -> bool {
        self.config.notifications.play_sound
    }

    /// Get the web URL for a messenger by ID
    pub fn get_messenger_url(&self, messenger_id: &str) -> Option<String> {
        self.config
            .enabled_messengers
            .iter()
            .find(|m| m.id == messenger_id)
            .map(|m| m.web_url.clone())
    }

    /// Get the display name for a messenger
    pub fn get_messenger_name(&self, messenger_id: &str) -> String {
        self.config
            .enabled_messengers
            .iter()
            .find(|m| m.id == messenger_id)
            .map(|m| m.name.clone())
            .unwrap_or_else(|| messenger_id.to_string())
    }

    /// Format notification summary for display
    pub fn format_summary<'a>(&self, data: &'a NotificationData) -> &'a str {
        if data.title.is_empty() {
            &data.app_name
        } else {
            &data.title
        }
    }

    /// Format notification body for display (safely handles multi-byte UTF-8)
    pub fn format_body(&self, data: &NotificationData) -> String {
        const MAX_LEN: usize = 200;
        let char_count = data.text.chars().count();
        if char_count > MAX_LEN {
            let truncated: String = data.text.chars().take(MAX_LEN).collect();
            format!("{}...", truncated)
        } else {
            data.text.clone()
        }
    }

    /// Build notification actions
    pub fn build_actions(&self, messenger_id: &str) -> Vec<NotificationAction> {
        vec![
            NotificationAction {
                id: "open".to_string(),
                label: "Open".to_string(),
                messenger_id: messenger_id.to_string(),
            },
            NotificationAction {
                id: "dismiss".to_string(),
                label: "Dismiss".to_string(),
                messenger_id: messenger_id.to_string(),
            },
        ]
    }
}

/// Action that can be taken on a notification
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NotificationAction {
    pub id: String,
    pub label: String,
    pub messenger_id: String,
}

/// Check if a package is a known messaging app
#[allow(dead_code)]
pub fn is_messaging_app(package: &str) -> bool {
    MessengerType::from_package(package) != MessengerType::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_messenger_type_from_package() {
        assert_eq!(
            MessengerType::from_package("com.google.android.apps.messaging"),
            MessengerType::GoogleMessages
        );
        assert_eq!(
            MessengerType::from_package("com.whatsapp"),
            MessengerType::WhatsApp
        );
        assert_eq!(
            MessengerType::from_package("org.telegram.messenger"),
            MessengerType::Telegram
        );
        assert_eq!(
            MessengerType::from_package("unknown.app"),
            MessengerType::Unknown
        );
    }

    #[test]
    fn test_is_messaging_app() {
        assert!(is_messaging_app("com.google.android.apps.messaging"));
        assert!(is_messaging_app("com.whatsapp"));
        assert!(!is_messaging_app("com.spotify.music"));
    }

    #[test]
    fn test_notification_handler() {
        let config = Config::default();
        let handler = NotificationHandler::new(config);

        let data = NotificationData::new(
            "com.google.android.apps.messaging".to_string(),
            "Messages".to_string(),
            "John".to_string(),
            "Hello!".to_string(),
            "device-1".to_string(),
        );

        let result = handler.handle_notification(&data);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "google-messages");
    }

    #[test]
    fn test_notification_handler_disabled() {
        let mut config = Config::default();
        config.toggle_messenger("google-messages", false);

        let handler = NotificationHandler::new(config);

        let data = NotificationData::new(
            "com.google.android.apps.messaging".to_string(),
            "Messages".to_string(),
            "John".to_string(),
            "Hello!".to_string(),
            "device-1".to_string(),
        );

        let result = handler.handle_notification(&data);
        assert!(result.is_none());
    }

    #[test]
    fn test_format_body_truncation() {
        let config = Config::default();
        let handler = NotificationHandler::new(config);

        let long_text = "a".repeat(300);
        let data = NotificationData {
            app_package: "test".to_string(),
            app_name: "Test".to_string(),
            title: "Title".to_string(),
            text: long_text,
            conversation_id: None,
            timestamp: 0,
            device_id: "device".to_string(),
            icon_data: None,
        };

        let formatted = handler.format_body(&data);
        assert!(formatted.len() < 210); // 200 + "..."
        assert!(formatted.ends_with("..."));
    }

    #[test]
    fn test_format_body_utf8_safe() {
        let config = Config::default();
        let handler = NotificationHandler::new(config);

        // Multi-byte UTF-8 characters (emoji + Japanese)
        let multibyte_text = "🎉".repeat(100) + &"こんにちは".repeat(50);
        let data = NotificationData {
            app_package: "test".to_string(),
            app_name: "Test".to_string(),
            title: "Title".to_string(),
            text: multibyte_text,
            conversation_id: None,
            timestamp: 0,
            device_id: "device".to_string(),
            icon_data: None,
        };

        // Should not panic on multi-byte UTF-8 boundaries
        let formatted = handler.format_body(&data);
        assert!(formatted.ends_with("..."));
        assert_eq!(formatted.chars().count(), 203); // 200 chars + "..."
    }

    #[test]
    fn test_messenger_shortcut_indices() {
        assert_eq!(MessengerType::GoogleMessages.shortcut_index(), Some(1));
        assert_eq!(MessengerType::WhatsApp.shortcut_index(), Some(2));
        assert_eq!(MessengerType::Telegram.shortcut_index(), Some(3));
        assert_eq!(MessengerType::Signal.shortcut_index(), Some(4));
        assert_eq!(MessengerType::Discord.shortcut_index(), Some(5));
        assert_eq!(MessengerType::Slack.shortcut_index(), Some(6));
        assert_eq!(MessengerType::Unknown.shortcut_index(), None);
    }

    #[test]
    fn test_signal_url_updated() {
        assert_eq!(
            MessengerType::Signal.web_url(),
            "https://signal.org/download/"
        );
    }
}
