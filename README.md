# cosmic-applet-kdeconnect

A native implementation of KDE Connect for COSMIC Desktop, written in Rust.

## Overview

`cosmic-applet-kdeconnect` provides seamless integration between your Android/iOS devices and COSMIC Desktop, enabling device synchronization, file sharing, notification mirroring, and remote control capabilities.

This project consists of:
- **Protocol Library**: Pure Rust implementation of the KDE Connect protocol
- **COSMIC Applet**: Panel/dock applet for quick access to connected devices
- **Full Application**: Comprehensive device management and configuration
- **Background Daemon**: Service for maintaining device connections

## Features

### Current Status: 🚧 In Development

#### Completed ✅
- [x] Core Protocol Library (v7)
- [x] Device State Management
- [x] TLS Certificate Generation
- [x] Packet Serialization/Deserialization
- [x] Plugin Architecture
  - [x] Ping Plugin
  - [x] Battery Plugin
  - [x] Notification Plugin
  - [x] Share Plugin (file/text/URL)
  - [x] Clipboard Plugin
  - [x] MPRIS Plugin (media control)
- [x] Background Daemon Service
- [x] COSMIC Panel Applet with Device List UI
- [x] Comprehensive Test Suite (91 tests)
- [x] CI/CD Pipeline with GitHub Actions
- [x] Integration Tests

#### In Progress 🔨
- [ ] Device Discovery (UDP broadcast)
- [ ] Active Pairing Flow
- [ ] Daemon ↔ Applet Communication
- [ ] TLS Connection Handling
- [ ] Plugin Packet Routing

#### Planned 📋
- [ ] File Transfer Execution
- [ ] Notification Mirroring
- [ ] Clipboard Synchronization
- [ ] Battery Status Display
- [ ] Media Player Control Integration
- [ ] Remote Input
- [ ] SMS Messaging
- [ ] Run Commands
- [ ] Bluetooth Transport

### Planned Features

- [ ] COSMIC Files integration
- [ ] COSMIC Notifications integration
- [ ] Per-device plugin configuration
- [ ] Network optimization
- [ ] Multiple device support
- [ ] Encryption key management

## Architecture

```
cosmic-applet-kdeconnect/
├── kdeconnect-protocol/          # Core protocol library
│   ├── src/
│   │   ├── lib.rs                # Public API
│   │   ├── discovery.rs          # Device discovery via UDP/mDNS
│   │   ├── pairing.rs            # TLS pairing and certificates
│   │   ├── packet.rs             # Packet serialization/deserialization
│   │   ├── device.rs             # Device state management
│   │   ├── transport/            # Network and Bluetooth transports
│   │   └── plugins/              # Plugin implementations
│   │       ├── battery.rs
│   │       ├── clipboard.rs
│   │       ├── notification.rs
│   │       ├── share.rs
│   │       └── ...
│   └── Cargo.toml
├── cosmic-applet-kdeconnect/     # COSMIC panel applet
│   ├── src/
│   │   └── main.rs               # Applet implementation
│   ├── data/
│   │   └── cosmic-applet-kdeconnect.desktop
│   └── Cargo.toml
├── cosmic-kdeconnect/            # Full desktop application
│   ├── src/
│   │   └── main.rs               # Application implementation
│   └── Cargo.toml
├── kdeconnect-daemon/            # Background service
│   ├── src/
│   │   └── main.rs               # Daemon implementation
│   └── Cargo.toml
└── Cargo.toml                     # Workspace configuration
```

## Technology Stack

- **Language**: Rust 🦀
- **GUI Framework**: [libcosmic](https://github.com/pop-os/libcosmic) (based on iced)
- **Async Runtime**: tokio
- **Network**: tokio/async-std, rustls for TLS
- **DBus**: zbus for system integration
- **Serialization**: serde + serde_json

## Prerequisites

### System Requirements

- COSMIC Desktop Environment
- Rust 1.70+ and Cargo
- Just command runner
- NixOS (recommended) or Linux with development libraries

### Required Libraries

- libxkbcommon-dev
- libwayland-dev
- libdbus-1-dev
- libssl-dev
- libfontconfig-dev
- libfreetype-dev
- pkg-config

## Development Setup

### NixOS (Recommended)

```bash
# Clone the repository
git clone https://github.com/yourusername/cosmic-applet-kdeconnect.git
cd cosmic-applet-kdeconnect

# Enter development shell
nix develop

# Build the project
just build

# Run tests
just test
```

### Other Linux Distributions

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install just
cargo install just

# Install system dependencies (Ubuntu/Debian)
sudo apt install libxkbcommon-dev libwayland-dev libdbus-1-dev \
                 libssl-dev libfontconfig-dev libfreetype-dev pkg-config

# Clone and build
git clone https://github.com/yourusername/cosmic-applet-kdeconnect.git
cd cosmic-applet-kdeconnect
just build
```

## Building

```bash
# Build all components
just build

# Build only the applet
just build-applet

# Build only the protocol library
just build-protocol

# Build with optimizations
just build-release
```

## Installation

```bash
# Install all components
sudo just install

# Install only the applet
sudo just install-applet

# For NixOS users, add to configuration.nix:
# environment.systemPackages = [ pkgs.cosmic-applet-kdeconnect ];
```

## Usage

### Setting Up

1. **Install KDE Connect on your mobile device**:
   - Android: [Google Play](https://play.google.com/store/apps/details?id=org.kde.kdeconnect_tp) or [F-Droid](https://f-droid.org/packages/org.kde.kdeconnect_tp/)
   - iOS: [App Store](https://apps.apple.com/app/kde-connect/id1580245991)

2. **Launch the applet**:
   - Add "KDE Connect" applet to your COSMIC panel via Settings → Panel → Applets

3. **Pair your device**:
   - Open KDE Connect on your mobile device
   - Click the applet icon in the panel
   - Select your device and click "Pair"
   - Accept the pairing request on your mobile device

### Firewall Configuration

KDE Connect requires ports 1714-1764 (TCP and UDP) to be open:

```bash
# For firewalld
sudo firewall-cmd --zone=public --permanent --add-port=1714-1764/tcp
sudo firewall-cmd --zone=public --permanent --add-port=1714-1764/udp
sudo firewall-cmd --reload

# For ufw
sudo ufw allow 1714:1764/tcp
sudo ufw allow 1714:1764/udp
```

### NixOS Firewall

Add to your `configuration.nix`:

```nix
networking.firewall = {
  allowedTCPPortRanges = [
    { from = 1714; to = 1764; }
  ];
  allowedUDPPortRanges = [
    { from = 1714; to = 1764; }
  ];
};
```

## Development

### Project Structure

The project uses a Cargo workspace with multiple crates:

- **kdeconnect-protocol**: Core protocol implementation (library)
- **cosmic-applet-kdeconnect**: Panel applet (binary)
- **cosmic-kdeconnect**: Full application (binary)
- **kdeconnect-daemon**: Background service (binary)

### Adding New Plugins

Plugins follow the KDE Connect plugin architecture:

```rust
// kdeconnect-protocol/src/plugins/example.rs
use crate::packet::Packet;
use async_trait::async_trait;

#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    async fn handle_packet(&mut self, packet: Packet) -> Result<(), Error>;
    async fn send_packet(&self, packet: Packet) -> Result<(), Error>;
}
```

### Testing

```bash
# Run all tests
just test

# Run protocol tests only
cargo test -p kdeconnect-protocol

# Run with verbose output
just test-verbose

# Test device discovery (requires network)
just test-discovery
```

### Code Quality

```bash
# Format code
just fmt

# Lint code
just lint

# Check for security issues
just audit
```

## Contributing

Contributions are welcome! Please see:
- [CONTRIBUTING.md](CONTRIBUTING.md) - Development workflow and guidelines
- [ACCEPTANCE_CRITERIA.md](ACCEPTANCE_CRITERIA.md) - Quality standards and definition of done

All contributions must meet the acceptance criteria to ensure consistent quality.

### Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Set up git hooks: `just setup` or `just install-hooks`
4. Make your changes
5. Git hooks will automatically:
   - Format your code (`cargo fmt`)
   - Run linting checks (`cargo clippy`)
   - Run tests (`cargo test`)
   - Enforce commit message format
6. Commit your changes (`git commit -m 'feat(scope): add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

**Note**: Git hooks automatically check code quality. See [hooks/README.md](hooks/README.md) for details.

## Protocol Compatibility

This implementation follows the KDE Connect protocol specification version 7/8.

**Compatible with:**
- KDE Connect Desktop (Linux, Windows, macOS)
- KDE Connect Android
- KDE Connect iOS
- GSConnect (GNOME)
- Valent (GTK)

**Protocol Documentation:**
- [KDE Connect Protocol](https://invent.kde.org/network/kdeconnect-kde)
- [Valent Protocol Reference](https://valent.andyholmes.ca/documentation/protocol.html)

## Resources

- [COSMIC Desktop](https://system76.com/cosmic)
- [libcosmic Documentation](https://pop-os.github.io/libcosmic-book/)
- [KDE Connect](https://kdeconnect.kde.org/)
- [KDE Connect Android](https://invent.kde.org/network/kdeconnect-android)

## License

This project is licensed under the GNU General Public License v3.0 or later - see the [LICENSE](LICENSE) file for details.

KDE Connect is a trademark of KDE e.V.

## Acknowledgments

- **KDE Connect Team** for the original protocol and applications
- **System76** for COSMIC Desktop and libcosmic
- **GSConnect/Valent** developers for implementation insights
- All contributors to the Rust and COSMIC ecosystems

## Status & Roadmap

### Current Phase: Foundation (Q1 2026)
- [x] Project structure
- [x] Development environment setup
- [ ] Core protocol implementation
- [ ] Device discovery
- [ ] TLS pairing

### Phase 2: Basic Functionality (Q2 2026)
- [ ] Basic applet UI
- [ ] File sharing
- [ ] Notification sync
- [ ] Battery status

### Phase 3: Advanced Features (Q3 2026)
- [ ] Clipboard sync
- [ ] Media control
- [ ] Remote input
- [ ] Bluetooth support

### Phase 4: Polish & Release (Q4 2026)
- [ ] Full COSMIC integration
- [ ] Performance optimization
- [ ] Documentation
- [ ] Public release

## Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/cosmic-applet-kdeconnect/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/cosmic-applet-kdeconnect/discussions)
- **COSMIC Community**: [Pop!_OS Mattermost](https://chat.pop-os.org/)

## Security

Found a security vulnerability? Please email security@yourproject.org instead of opening a public issue.

---

**Note**: This project is under active development. Features and APIs may change.
