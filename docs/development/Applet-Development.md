# COSMIC Applet Development Skill

## Purpose

This skill provides guidance for developing COSMIC Desktop applets using libcosmic and Rust.

## When to Use This Skill

Use this skill when:
- Creating COSMIC panel or dock applets
- Implementing applet UI components
- Working with libcosmic widgets
- Handling applet popup windows
- Integrating with COSMIC Desktop
- Debugging applet behavior

## Applet Fundamentals

### What is a COSMIC Applet?

A COSMIC applet is a **standalone application** that:
- Runs in its own process
- Displays in the COSMIC panel or dock
- Has a transparent, undecorated main window
- Can show popup windows for expanded UI
- Reads configuration from environment variables
- Integrates with COSMIC theming

**Key Differences from Extensions:**
- Not JavaScript-based (unlike GNOME extensions)
- Separate process (better isolation and security)
- Uses same toolkit as full apps (libcosmic/iced)
- Can be installed as regular packages

## Basic Applet Structure

### Cargo.toml Configuration

```toml
[package]
name = "cosmic-applet-kdeconnect"
version = "0.1.0"
edition = "2021"

[dependencies]
libcosmic = { git = "https://github.com/pop-os/libcosmic", features = ["applet"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }

# For software rendering (lower memory usage)
[dependencies.libcosmic]
git = "https://github.com/pop-os/libcosmic"
features = ["applet"]
default-features = false
```

### Desktop Entry

Create `data/cosmic-applet-kdeconnect.desktop`:

```desktop
[Desktop Entry]
Name=KDE Connect
Name[es]=KDE Connect
Comment=Connect and sync your devices
Comment[es]=Conecta y sincroniza tus dispositivos
Type=Application
Exec=cosmic-applet-kdeconnect
Terminal=false
Categories=COSMIC;
Keywords=COSMIC;Iced;KDEConnect;Sync;Phone;

# Essential applet keys
NoDisplay=true
X-CosmicApplet=true
X-CosmicHoverPopup=Auto
X-OverflowPriority=10

# Translators: Icon file name - do not translate
Icon=phone-symbolic
```

**Key Fields:**
- `NoDisplay=true` - Hides from app launcher
- `X-CosmicApplet=true` - Marks as applet
- `X-CosmicHoverPopup=Auto` - Popup on hover behavior
- `X-OverflowPriority=10` - Priority when panel is full

## Minimal Applet Implementation

```rust
use cosmic::{
    app::{Core, Message as CosmicMessage},
    applet::{self, Context},
    iced::{
        self,
        widget::{column, container, row, text},
        window, Alignment, Length,
    },
    iced_runtime::Command,
    iced_style::application,
    Element, Theme,
};

fn main() -> cosmic::iced::Result {
    // Launch applet instead of app
    applet::run::<KdeConnectApplet>(())
}

struct KdeConnectApplet {
    core: Core,
    popup: Option<window::Id>,
    icon_name: String,
    device_count: usize,
}

#[derive(Debug, Clone)]
enum Message {
    TogglePopup,
    DeviceCountChanged(usize),
}

impl cosmic::Application for KdeConnectApplet {
    type Message = Message;
    type Executor = cosmic::executor::Default;
    type Flags = ();
    const APP_ID: &'static str = "io.github.olafkfreund.CosmicExtAppletConnect";

    fn init(core: Core, _flags: Self::Flags) -> (Self, Command<Message>) {
        let applet = Self {
            core,
            popup: None,
            icon_name: "phone-symbolic".to_string(),
            device_count: 0,
        };
        (applet, Command::none())
    }

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::TogglePopup => {
                if let Some(id) = self.popup.take() {
                    // Close existing popup
                    return cosmic::iced::platform_specific::shell::commands::popup::destroy_popup(id);
                } else {
                    // Create new popup
                    let new_id = window::Id::unique();
                    self.popup = Some(new_id);

                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        Some((400, 300)), // Width, height
                        None,
                        None,
                    );

                    popup_settings.positioner.size_limits = cosmic::iced::Limits::NONE
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_width(600.0)
                        .max_height(800.0);

                    return cosmic::iced::platform_specific::shell::commands::popup::get_popup(popup_settings);
                }
            }
            Message::DeviceCountChanged(count) => {
                self.device_count = count;
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        // Main applet view (what appears in panel)
        self.core
            .applet
            .icon_button(&self.icon_name)
            .on_press_down(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, id: window::Id) -> Element<Self::Message> {
        // Popup window view
        let content = if Some(id) == self.popup {
            column![
                text("KDE Connect").size(20),
                text(format!("{} devices connected", self.device_count)),
                // Add more content here
            ]
            .spacing(10)
            .padding(20)
        } else {
            column![]
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
```

## Applet Context

### Reading Panel Configuration

```rust
let context = applet::Context::default();

// Check panel orientation
if context.is_horizontal() {
    // Panel is on top or bottom
} else {
    // Panel is on left or right
}

// Get panel anchor
match context.anchor {
    applet::PanelAnchor::Top => { /* ... */ }
    applet::PanelAnchor::Bottom => { /* ... */ }
    applet::PanelAnchor::Left => { /* ... */ }
    applet::PanelAnchor::Right => { /* ... */ }
}

// Get suggested size
let suggested_size = context.suggested_size(false); // false = compact mode
```

### Panel-Aware Dimensions

```rust
impl KdeConnectApplet {
    fn icon_size(&self) -> u16 {
        // Get appropriate icon size for current panel
        self.core.applet.suggested_size(false).0 as u16
    }

    fn padding(&self) -> u16 {
        // Get appropriate padding
        self.core.applet.suggested_padding(false).0 as u16
    }
}
```

## Advanced UI Components

### Custom Icon Button

```rust
use cosmic::widget::icon;

fn device_icon_button<'a>(
    icon_name: &'a str,
    device_name: &'a str,
    battery: Option<u8>,
) -> Element<'a, Message> {
    let icon_widget = icon::from_name(icon_name).size(32);
    
    let content = if let Some(level) = battery {
        row![
            icon_widget,
            column![
                text(device_name).size(14),
                text(format!("{}%", level)).size(12),
            ]
            .spacing(2)
        ]
        .spacing(8)
        .align_items(Alignment::Center)
    } else {
        row![
            icon_widget,
            text(device_name).size(14),
        ]
        .spacing(8)
        .align_items(Alignment::Center)
    };

    cosmic::widget::button(content)
        .style(cosmic::theme::Button::AppletIcon)
        .on_press(Message::DeviceSelected(device_name.to_string()))
        .into()
}
```

### List of Devices

```rust
fn device_list<'a>(devices: &'a [Device]) -> Element<'a, Message> {
    let items: Vec<Element<'a, Message>> = devices
        .iter()
        .map(|device| {
            cosmic::widget::settings::item(
                &device.name,
                device_status_widget(device)
            )
            .apply(cosmic::widget::container)
            .style(cosmic::theme::Container::List)
            .into()
        })
        .collect();

    column(items)
        .spacing(4)
        .into()
}

fn device_status_widget<'a>(device: &'a Device) -> Element<'a, Message> {
    let status_text = if device.is_connected {
        "Connected"
    } else {
        "Disconnected"
    };

    row![
        text(status_text).size(12),
        cosmic::widget::button::icon(
            icon::from_name("send-symbolic").size(16)
        )
        .on_press(Message::SendFile(device.id.clone()))
    ]
    .spacing(8)
    .align_items(Alignment::Center)
    .into()
}
```

### Popup with Scrollable Content

```rust
fn view_window(&self, id: window::Id) -> Element<Self::Message> {
    if Some(id) != self.popup {
        return column![].into();
    }

    let header = row![
        text("KDE Connect").size(20),
        cosmic::widget::horizontal_space(Length::Fill),
        cosmic::widget::button::icon(
            icon::from_name("settings-symbolic").size(16)
        )
        .on_press(Message::OpenSettings)
    ]
    .align_items(Alignment::Center);

    let device_list = cosmic::widget::scrollable(
        column(
            self.devices
                .iter()
                .map(|device| device_item(device).into())
                .collect()
        )
        .spacing(4)
    )
    .height(Length::Fill);

    let content = column![header, device_list]
        .spacing(12)
        .padding(16);

    cosmic::widget::container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(cosmic::theme::Container::Background)
        .into()
}
```

## State Management

### Subscription for Updates

```rust
use cosmic::iced::Subscription;

impl cosmic::Application for KdeConnectApplet {
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch([
            // Device discovery updates
            device_discovery_subscription(),
            // Connection status updates
            connection_status_subscription(),
            // Notification updates
            notification_subscription(),
        ])
    }
}

fn device_discovery_subscription() -> Subscription<Message> {
    cosmic::iced::subscription::channel(
        std::any::TypeId::of::<DeviceDiscovery>(),
        100,
        |mut output| async move {
            let mut receiver = start_device_discovery().await;
            
            loop {
                if let Some(device) = receiver.recv().await {
                    let _ = output.send(Message::DeviceDiscovered(device)).await;
                }
            }
        }
    )
}
```

### Background Tasks

```rust
impl KdeConnectApplet {
    fn refresh_devices(&self) -> Command<Message> {
        Command::perform(
            async {
                // Simulate async operation
                tokio::time::sleep(Duration::from_millis(100)).await;
                vec![
                    Device { name: "Phone".to_string(), connected: true },
                    Device { name: "Tablet".to_string(), connected: false },
                ]
            },
            Message::DevicesRefreshed
        )
    }
}
```

## Theming and Styling

### Using COSMIC Theme

```rust
use cosmic::theme;

// Container styles
cosmic::widget::container(content)
    .style(theme::Container::Background)  // Standard background
    .style(theme::Container::Card)        // Card style
    .style(theme::Container::List)        // List item style

// Button styles
cosmic::widget::button(content)
    .style(theme::Button::Standard)       // Standard button
    .style(theme::Button::AppletIcon)     // Icon button for applet
    .style(theme::Button::Destructive)    // Destructive action
```

### Custom Colors

```rust
fn status_color(connected: bool) -> cosmic::iced::Color {
    if connected {
        cosmic::iced::Color::from_rgb(0.3, 0.8, 0.3) // Green
    } else {
        cosmic::iced::Color::from_rgb(0.6, 0.6, 0.6) // Gray
    }
}
```

## Panel Integration

### Handling Panel Events

```rust
#[derive(Debug, Clone)]
enum Message {
    // Panel triggered messages
    PanelSizeChanged(u32),
    PanelAnchorChanged(applet::PanelAnchor),
    ThemeChanged(Theme),
    // Regular messages
    TogglePopup,
}

impl cosmic::Application for KdeConnectApplet {
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::PanelSizeChanged(size) => {
                // Adjust UI based on panel size
                self.icon_size = (size as f32 * 0.6) as u16;
            }
            Message::ThemeChanged(theme) => {
                self.core.set_theme(theme);
            }
            // ... other messages
        }
        Command::none()
    }
}
```

## Resource Management

### Efficient Icon Loading

```rust
use cosmic::widget::icon;
use std::collections::HashMap;

struct IconCache {
    cache: HashMap<String, icon::Handle>,
}

impl IconCache {
    fn get_or_load(&mut self, name: &str, size: u16) -> icon::Handle {
        let key = format!("{}:{}", name, size);
        
        self.cache
            .entry(key)
            .or_insert_with(|| {
                icon::from_name(name)
                    .size(size)
                    .handle()
            })
            .clone()
    }
}
```

### Lazy Loading Content

```rust
enum ContentState {
    NotLoaded,
    Loading,
    Loaded(Vec<Device>),
    Error(String),
}

impl KdeConnectApplet {
    fn view_popup(&self) -> Element<Message> {
        match &self.content_state {
            ContentState::NotLoaded | ContentState::Loading => {
                column![
                    text("Loading devices..."),
                    cosmic::widget::spinner(),
                ]
                .into()
            }
            ContentState::Loaded(devices) => {
                device_list(devices)
            }
            ContentState::Error(err) => {
                column![
                    text("Error loading devices"),
                    text(err).size(12),
                ]
                .into()
            }
        }
    }
}
```

## DBus Integration

### Notification Integration

```rust
use zbus::{Connection, dbus_proxy};

#[dbus_proxy(
    interface = "org.freedesktop.Notifications",
    default_service = "org.freedesktop.Notifications",
    default_path = "/org/freedesktop/Notifications"
)]
trait Notifications {
    fn notify(
        &self,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: &[&str],
        hints: std::collections::HashMap<&str, zbus::zvariant::Value<'_>>,
        expire_timeout: i32,
    ) -> zbus::Result<u32>;
}

async fn send_notification(title: &str, body: &str) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::session().await?;
    let proxy = NotificationsProxy::new(&connection).await?;
    
    proxy.notify(
        "KDE Connect",
        0,
        "phone-symbolic",
        title,
        body,
        &[],
        HashMap::new(),
        5000,
    ).await?;
    
    Ok(())
}
```

## Debugging and Testing

### Enable Logging

```rust
use tracing::{info, debug, warn, error};
use tracing_subscriber;

fn main() -> cosmic::iced::Result {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
        )
        .init();
    
    info!("Starting KDE Connect applet");
    
    applet::run::<KdeConnectApplet>(())
}
```

### Run Applet in Development

```bash
# With debug logging
RUST_LOG=debug cargo run

# With specific module logging
RUST_LOG=cosmic_applet_kdeconnect=trace cargo run

# Simulate panel environment variables
COSMIC_PANEL_SIZE=48 \
COSMIC_PANEL_ANCHOR=Top \
cargo run
```

### Testing Popup Behavior

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popup_toggle() {
        let core = Core::default();
        let mut applet = KdeConnectApplet::init(core, ()).0;
        
        // Initially no popup
        assert!(applet.popup.is_none());
        
        // Toggle popup
        applet.update(Message::TogglePopup);
        assert!(applet.popup.is_some());
        
        // Toggle again
        applet.update(Message::TogglePopup);
        assert!(applet.popup.is_none());
    }
}
```

## Installation

### Justfile for Building

```just
# Build applet
build-applet:
    cargo build --release -p cosmic-applet-kdeconnect

# Install applet
install-applet: build-applet
    install -Dm755 target/release/cosmic-applet-kdeconnect \
        {{DESTDIR}}/usr/bin/cosmic-applet-kdeconnect
    install -Dm644 data/cosmic-applet-kdeconnect.desktop \
        {{DESTDIR}}/usr/share/applications/cosmic-applet-kdeconnect.desktop

# Run applet in development
run-applet:
    cargo run -p cosmic-applet-kdeconnect
```

## Best Practices

1. **Keep applet UI minimal** - main view should be just an icon
2. **Use popups for details** - extended UI goes in popup windows
3. **Be panel-size aware** - adapt to different panel sizes
4. **Handle disconnections** - gracefully handle panel restarts
5. **Use async operations** - keep UI responsive
6. **Cache resources** - minimize repeated allocations
7. **Follow COSMIC design** - use standard widgets and patterns
8. **Test on different panels** - top, bottom, left, right

## Common Patterns

### Device Status Indicator

```rust
fn status_indicator(connected: bool) -> Element<Message> {
    let color = if connected {
        cosmic::iced::Color::from_rgb(0.3, 0.8, 0.3)
    } else {
        cosmic::iced::Color::from_rgb(0.8, 0.3, 0.3)
    };
    
    cosmic::widget::container(text(""))
        .width(Length::Fixed(8.0))
        .height(Length::Fixed(8.0))
        .style(move |_theme| {
            cosmic::widget::container::Appearance {
                background: Some(cosmic::iced::Background::Color(color)),
                border: cosmic::iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}
```

### Badge Counter

```rust
fn icon_with_badge(icon_name: &str, count: usize) -> Element<Message> {
    cosmic::widget::layer_container(
        icon::from_name(icon_name).size(32)
    )
    .layer(
        if count > 0 {
            Some(
                cosmic::widget::container(
                    text(format!("{}", count)).size(10)
                )
                .style(cosmic::theme::Container::Badge)
                .into()
            )
        } else {
            None
        }
    )
    .into()
}
```

## Resources

- [libcosmic Book](https://pop-os.github.io/libcosmic-book/)
- [libcosmic Examples](https://github.com/pop-os/libcosmic/tree/master/examples)
- [COSMIC Applets Source](https://github.com/pop-os/cosmic-applets)
- [iced Documentation](https://docs.rs/iced/)
