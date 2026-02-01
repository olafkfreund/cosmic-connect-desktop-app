<p align="center">
  <img src="connect_logo.png" alt="COSMIC Connect Logo" width="200"/>
</p>

# COSMIC Connect

A modern, cross-platform device connectivity solution for COSMIC Desktop, written in Rust with 70%+ code sharing between desktop and mobile platforms.

## Overview

**COSMIC Connect** provides seamless integration between your Android devices and COSMIC Desktop, enabling device synchronization, file sharing, notification mirroring, clipboard sync, remote control capabilities, and advanced desktop-to-desktop collaboration features.

This project is part of a **multi-platform ecosystem**:

- **[cosmic-connect-core](https://github.com/olafkfreund/cosmic-connect-core)** - Shared Rust library (protocol, TLS, plugins)
- **[cosmic-connect-desktop-app](https://github.com/olafkfreund/cosmic-connect-desktop-app)** - This repository (COSMIC Desktop)
- **[cosmic-connect-android](https://github.com/olafkfreund/cosmic-connect-android)** - Android app with Kotlin FFI bindings

### Key Innovations

- **70%+ Code Sharing** - Unified Rust core shared between desktop and Android
- **Protocol Independence** - CConnect protocol (v7/8 compatible) with unique port 1816
- **Side-by-Side Operation** - Can run alongside KDE Connect without conflicts
- **No OpenSSL** - Modern rustls-based TLS (better cross-compilation)
- **FFI Bindings** - Kotlin/Swift support via uniffi-rs
- **Modern Async** - Tokio-based concurrent architecture
- **COSMIC Design Compliance** - Hierarchical text, theme integration, WCAG AA+ accessibility

## Architecture

See **[Architecture Documentation](docs/architecture/Architecture.md)** for comprehensive documentation.

```
cosmic-connect-core (Shared Library)
├── Protocol v7 implementation
├── TLS/crypto layer (rustls)
├── Network & discovery
├── Plugin system
└── FFI bindings (uniffi-rs) ──┐
                                │
                                ├──→ Desktop (This Repo)
                                │    ├── cosmic-connect-protocol
                                │    ├── cosmic-connect-daemon
                                │    └── cosmic-applet-connect
                                │
                                └──→ Android App
                                     └── Kotlin via FFI
```

## Features

### Status: Production Ready

**Version:** 0.1.0
**Protocol:** CConnect v7/8 (KDE Connect compatible)
**Discovery Port:** 1816
**Plugin Count:** 22 plugins (12 core + 10 advanced)

#### Core Features

- **Device Discovery** - UDP broadcast + mDNS service discovery
- **Secure Pairing** - TLS certificate exchange with user verification
- **Connection Management** - Auto-reconnect, exponential backoff, socket replacement
- **Background Daemon** - Systemd service with DBus interface
- **COSMIC Panel Applet** - Modern UI with device cards, details view, and transfer queue
- **Per-Device Settings** - Plugin enable/disable per device

#### Implemented Plugins

| Category | Plugin | Status | Description |
|----------|--------|--------|-------------|
| **Comm** | Ping | ✅ | Test connectivity |
| | Battery | ✅ | Monitor battery & charge state |
| | Notification | ✅ | Mirror notifications |
| | Share | ✅ | File, text, and URL sharing |
| | Clipboard | ✅ | Bidirectional clipboard sync |
| | Telephony | ✅ | Call & SMS notifications |
| | Contacts | ✅ | Contact synchronization |
| **Control** | MPRIS | ✅ | Media player remote control |
| | Remote Input | ✅ | Mouse & keyboard control |
| | Run Command | ✅ | Execute desktop commands |
| | Find My Phone | ✅ | Ring remote device |
| | Presenter | ✅ | Presentation control |
| **System** | System Monitor | ✅ | Remote CPU/RAM stats |
| | Lock | ✅ | Remote lock/unlock |
| | Power | ✅ | Shutdown/reboot/suspend |
| | Wake-on-LAN | ✅ | Wake sleeping devices |
| | Screenshot | ✅ | Capture remote screen |
| | Clipboard History | ✅ | Persistent history |
| **Files** | Network Share | ✅ | SFTP filesystem mounting |
| | File Sync | 🚧 | Automatic folder sync |
| **Adv** | Remote Desktop | ✅ | VNC screen sharing (Receiver) |
| | Screen Mirroring | 🚧 | H.264 streaming (In Progress) |

### Rich Notifications (Desktop to Android)

COSMIC Connect supports forwarding desktop notifications to connected Android devices with full rich content preservation. Notifications are captured via DBus using the freedesktop.org notification specification and transmitted as extended protocol packets.

#### Supported Rich Content

| Content Type | Description |
|-------------|-------------|
| **Images** | Notification images from `image-data` hint or file paths (resized to 256x256, PNG encoded) |
| **App Icons** | Application icons transmitted as base64-encoded PNG |
| **Urgency** | Three levels: Low (0), Normal (1), Critical (2) |
| **Categories** | Standard categories: `email`, `im.received`, `device`, `network`, etc. |
| **Actions** | Interactive buttons with ID/label pairs (Reply, Mark Read, etc.) |
| **HTML Body** | Rich text formatting preserved in `richBody` field |

#### Protocol Packet Format

Desktop notifications are sent as `cconnect.notification` packets:

```json
{
  "id": 1234567890,
  "type": "cconnect.notification",
  "body": {
    "id": "desktop-Thunderbird-1704067200000",
    "appName": "Thunderbird",
    "title": "New Email",
    "text": "You have a new message from Alice",
    "ticker": "Thunderbird: New Email - You have...",
    "isClearable": true,
    "time": "1704067200000",
    "silent": "false",
    "imageData": "<base64-png>",
    "appIcon": "<base64-png>",
    "urgency": 1,
    "category": "email",
    "actionButtons": [
      {"id": "reply", "label": "Reply"},
      {"id": "mark_read", "label": "Mark as Read"}
    ]
  }
}
```

#### Configuration Options

The notification listener is configured in the daemon configuration file:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | bool | `true` | Enable/disable notification forwarding |
| `excluded_apps` | string[] | `["CConnect", "cosmic-connect", "cosmic-notifications"]` | Apps to exclude (prevents loops) |
| `included_apps` | string[] | `[]` | Whitelist mode (empty = all non-excluded) |
| `include_transient` | bool | `true` | Forward transient notifications |
| `include_low_urgency` | bool | `true` | Forward low-priority notifications |
| `max_body_length` | number | `0` | Truncate body text (0 = no limit) |

#### Bidirectional Sync

- **Dismissal Sync**: Dismissing a notification on Android sends `isCancel: true` back to desktop
- **Action Invocation**: Tapping action buttons sends `cconnect.notification.action` packet with action ID
- **Request All**: Android can request all active notifications via `cconnect.notification.request`

### Recently Completed (Q1 2026)

- **UI Overhaul**: Modern card-based device list, detailed device view, and transfer queue.
- **Error Handling**: Centralized error reporting, user notifications for failures, and auto-recovery.
- **New Plugins**: Network Share (SFTP), Contacts (SQLite), Run Command, Remote Input.
- **Backend Stability**: Fixed DBus interface types, improved connection reliability with backoff.

## COSMIC Connect Manager

The **COSMIC Connect Manager** is a standalone window application for comprehensive device management, providing a full-featured interface beyond the panel applet.

### Features

- **Device List**: View all paired devices with real-time status indicators (Connected, Available, Offline)
- **Media Controls**: MPRIS-based media player remote control with playback, volume, and track navigation
- **File Transfers**: Monitor and manage file transfer queue with progress tracking
- **Plugin Settings**: Configure per-device plugin preferences and permissions

### Launching the Manager

**From the Panel Applet:**
Click the "Open Manager" button in the COSMIC Connect applet dropdown.

**From Command Line:**
```bash
# Open manager window
cosmic-connect-manager

# Open with a specific device selected
cosmic-connect-manager --device <device-id>
```

### Window Layout

The Manager uses a two-panel layout:

| Panel | Description |
|-------|-------------|
| **Sidebar** | Device list with status icons, search, and quick actions |
| **Content Area** | Device details, plugin controls, and transfer management |

The sidebar provides navigation between devices, while the content area displays context-specific controls based on the selected device and active plugins.

## Installation

### NixOS (Flake)

Add to your `flake.nix`:

```nix
{
  inputs.cosmic-connect.url = "github:olafkfreund/cosmic-connect-desktop-app";
  
  outputs = { self, nixpkgs, cosmic-connect, ... }:
    {
      nixosConfigurations.your-hostname = nixpkgs.lib.nixosSystem {
        modules = [
          cosmic-connect.nixosModules.default
          {
            services.cosmic-connect.enable = true;
            services.cosmic-connect.openFirewall = true;
          }
        ];
      };
    };
}
```

### Manual Installation

```bash
# Build release binaries
cargo build --release

# Install daemon
sudo install -Dm755 target/release/cosmic-connect-daemon /usr/local/bin/
sudo install -Dm644 cosmic-connect-daemon/cosmic-connect-daemon.service \
  /usr/lib/systemd/user/

# Install applet
sudo install -Dm755 target/release/cosmic-applet-connect /usr/local/bin/

# Enable and start daemon
systemctl --user enable --now cosmic-connect-daemon
```

## Documentation

- **[User Guide](docs/USER_GUIDE.md)** - Setup and usage instructions
- **[Architecture](docs/architecture/Architecture.md)** - System design
- **[Contributing](CONTRIBUTING.md)** - Development guide

## License

GNU General Public License v3.0 or later.