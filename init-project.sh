#!/usr/bin/env bash
# Project initialization script for cosmic-applet-kdeconnect

set -e

echo "🚀 Initializing cosmic-applet-kdeconnect project structure..."
echo ""

# Create directory structure
echo "📁 Creating directory structure..."

# Protocol library
mkdir -p kdeconnect-protocol/src/plugins
mkdir -p kdeconnect-protocol/src/transport
mkdir -p kdeconnect-protocol/tests

# Applet
mkdir -p cosmic-applet-kdeconnect/src
mkdir -p cosmic-applet-kdeconnect/data

# Full application
mkdir -p cosmic-kdeconnect/src
mkdir -p cosmic-kdeconnect/data

# Daemon
mkdir -p kdeconnect-daemon/src

echo "✅ Directory structure created"
echo ""

# Create Cargo.toml files for each crate
echo "📦 Creating Cargo.toml files..."

# Protocol library Cargo.toml
cat > kdeconnect-protocol/Cargo.toml << 'EOF'
[package]
name = "kdeconnect-protocol"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "KDE Connect protocol implementation in Rust"

[dependencies]
tokio = { workspace = true }
async-trait = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
rustls = { workspace = true }
tokio-rustls = { workspace = true }
rcgen = { workspace = true }
rustls-pemfile = { workspace = true }
mdns-sd = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
sha2 = { workspace = true }
hex = { workspace = true }

[dev-dependencies]
tokio-test = "0.4"
EOF

# Applet Cargo.toml
cat > cosmic-applet-kdeconnect/Cargo.toml << 'EOF'
[package]
name = "cosmic-applet-kdeconnect"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "KDE Connect applet for COSMIC Desktop"

[dependencies]
libcosmic = { workspace = true, features = ["applet"] }
kdeconnect-protocol = { path = "../kdeconnect-protocol" }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
anyhow = { workspace = true }

[[bin]]
name = "cosmic-applet-kdeconnect"
path = "src/main.rs"
EOF

# Full application Cargo.toml
cat > cosmic-kdeconnect/Cargo.toml << 'EOF'
[package]
name = "cosmic-kdeconnect"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "KDE Connect application for COSMIC Desktop"

[dependencies]
libcosmic = { workspace = true }
kdeconnect-protocol = { path = "../kdeconnect-protocol" }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
anyhow = { workspace = true }

[[bin]]
name = "cosmic-kdeconnect"
path = "src/main.rs"
EOF

# Daemon Cargo.toml
cat > kdeconnect-daemon/Cargo.toml << 'EOF'
[package]
name = "kdeconnect-daemon"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "KDE Connect background daemon"

[dependencies]
kdeconnect-protocol = { path = "../kdeconnect-protocol" }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
anyhow = { workspace = true }
dirs = { workspace = true }
toml = { workspace = true }
zbus = { workspace = true }

[[bin]]
name = "kdeconnect-daemon"
path = "src/main.rs"
EOF

echo "✅ Cargo.toml files created"
echo ""

# Create basic source files
echo "📝 Creating initial source files..."

# Protocol library lib.rs
cat > kdeconnect-protocol/src/lib.rs << 'EOF'
//! KDE Connect Protocol Implementation
//!
//! This library provides a pure Rust implementation of the KDE Connect protocol,
//! enabling device synchronization and communication between computers and mobile devices.

pub mod discovery;
pub mod pairing;
pub mod packet;
pub mod device;
pub mod plugins;
pub mod transport;

mod error;
pub use error::{ProtocolError, Result};

/// Protocol version we implement
pub const PROTOCOL_VERSION: u32 = 7;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_version() {
        assert_eq!(PROTOCOL_VERSION, 7);
    }
}
EOF

cat > kdeconnect-protocol/src/error.rs << 'EOF'
use thiserror::Error;

pub type Result<T> = std::result::Result<T, ProtocolError>;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TLS error: {0}")]
    Tls(#[from] rustls::Error),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Not paired")]
    NotPaired,

    #[error("Invalid packet: {0}")]
    InvalidPacket(String),

    #[error("Plugin error: {0}")]
    Plugin(String),
}
EOF

# Create placeholder files
touch kdeconnect-protocol/src/{discovery,pairing,packet,device}.rs
touch kdeconnect-protocol/src/plugins/mod.rs
touch kdeconnect-protocol/src/transport/mod.rs

# Applet main.rs
cat > cosmic-applet-kdeconnect/src/main.rs << 'EOF'
use cosmic::{
    app::Core,
    applet,
    iced::window,
    Application, Element,
};

fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt::init();
    applet::run::<KdeConnectApplet>(())
}

struct KdeConnectApplet {
    core: Core,
}

#[derive(Debug, Clone)]
enum Message {}

impl Application for KdeConnectApplet {
    type Message = Message;
    type Executor = cosmic::executor::Default;
    type Flags = ();
    const APP_ID: &'static str = "io.github.olafkfreund.CosmicExtAppletConnect";

    fn init(core: Core, _flags: Self::Flags) -> (Self, cosmic::iced::Command<Message>) {
        (Self { core }, cosmic::iced::Command::none())
    }

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn update(&mut self, _message: Self::Message) -> cosmic::iced::Command<Self::Message> {
        cosmic::iced::Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        self.core
            .applet
            .icon_button("phone-symbolic")
            .into()
    }
}
EOF

# Desktop entry
cat > cosmic-applet-kdeconnect/data/cosmic-applet-kdeconnect.desktop << 'EOF'
[Desktop Entry]
Name=KDE Connect
Comment=Connect and sync your devices
Type=Application
Exec=cosmic-applet-kdeconnect
Terminal=false
Categories=COSMIC;
NoDisplay=true
X-CosmicApplet=true
X-CosmicHoverPopup=Auto
X-OverflowPriority=10
Icon=phone-symbolic
EOF

# Full app main.rs
cat > cosmic-kdeconnect/src/main.rs << 'EOF'
use cosmic::{app::Core, Application};

fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt::init();
    cosmic::app::run::<KdeConnectApp>((), ())
}

struct KdeConnectApp {
    core: Core,
}

#[derive(Debug, Clone)]
enum Message {}

impl Application for KdeConnectApp {
    type Message = Message;
    type Executor = cosmic::executor::Default;
    type Flags = ();
    const APP_ID: &'static str = "io.github.olafkfreund.CosmicExtConnect.Manager";

    fn init(core: Core, _flags: Self::Flags) -> (Self, cosmic::iced::Command<Message>) {
        (Self { core }, cosmic::iced::Command::none())
    }

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn update(&mut self, _message: Self::Message) -> cosmic::iced::Command<Self::Message> {
        cosmic::iced::Command::none()
    }

    fn view(&self) -> cosmic::Element<Self::Message> {
        cosmic::widget::text("KDE Connect").into()
    }
}
EOF

# Daemon main.rs
cat > kdeconnect-daemon/src/main.rs << 'EOF'
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("Starting KDE Connect daemon...");
    
    // TODO: Initialize daemon
    
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");
    
    Ok(())
}
EOF

echo "✅ Initial source files created"
echo ""

echo "🎉 Project structure initialized successfully!"
echo ""
echo "Next steps:"
echo "  1. cd into the project directory"
echo "  2. Run 'nix develop' or 'nix-shell' to enter dev environment"
echo "  3. Run 'just build' to build the project"
echo "  4. Start implementing features!"
echo ""
echo "For more information, see:"
echo "  - README.md for project overview"
echo "  - .claude/claude.md for development context"
echo "  - .claude/skills/ for implementation guides"
echo ""
