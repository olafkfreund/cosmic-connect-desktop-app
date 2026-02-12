# COSMIC Connect Architecture

## Overview

COSMIC Connect implements a modern, cross-platform device connectivity solution using a shared Rust core library that enables 70%+ code sharing between desktop and mobile platforms.

## Repository Structure

```
cosmic-ext-connect-core/          # Shared Rust library (github.com/olafkfreund/cosmic-ext-connect-core)
├── protocol/                 # Core protocol types (Packet, Device, Identity)
├── network/                  # Discovery, TCP transport
├── crypto/                   # TLS, certificate management (rustls-based)
├── plugins/                  # Plugin system and implementations
└── ffi/                      # FFI bindings for Kotlin/Swift (uniffi-rs)

cosmic-connect-desktop-app/   # This repository (COSMIC Desktop)
├── cosmic-ext-connect-protocol/  # Desktop-specific protocol extensions
├── cosmic-ext-connect-daemon/    # Background daemon service
├── cosmic-ext-applet-connect/    # COSMIC panel applet (UI)
└── cosmic-connect/           # CLI tool

cosmic-connect-android/       # Android app (github.com/olafkfreund/cosmic-connect-android)
└── Uses cosmic-ext-connect-core via Kotlin FFI bindings
```

## Architecture Layers

### 1. Shared Core (`cosmic-ext-connect-core`)

**Location:** https://github.com/olafkfreund/cosmic-ext-connect-core

**Purpose:** Platform-agnostic KDE Connect protocol v7 implementation

**Key Components:**
- **Protocol:** NetworkPacket, Device identity, JSON serialization
- **Crypto:** TLS 1.2+ using rustls (no OpenSSL), certificate generation (rcgen)
- **Network:** mDNS discovery, TCP transport, async I/O (tokio)
- **Plugins:** Battery, Ping, Clipboard, MPRIS, Notification, Share, etc.
- **FFI:** uniffi-rs bindings for Kotlin and Swift

**Exports:**
```rust
// From cosmic-ext-connect-core
pub use crypto::{
    CertificateInfo,        // Certificate generation and management
    DeviceInfo,             // TLS device information
    TlsConfig,              // TLS configuration
    TlsConnection,          // Async TLS connection
    TlsServer,              // TLS server
    should_initiate_connection, // Connection priority logic
};
pub use {Packet, ProtocolError};
```

**Benefits:**
- ✅ 70%+ code sharing between platforms
- ✅ 100% protocol compatibility
- ✅ Bug fixes benefit all platforms simultaneously
- ✅ No OpenSSL dependency (better cross-compilation)
- ✅ Modern async Rust with tokio
- ✅ FFI bindings for Android (Kotlin) via uniffi-rs

### 2. Desktop Protocol Layer (`cosmic-ext-connect-protocol`)

**Purpose:** Desktop-specific protocol extensions and integrations

**Key Components:**
- Connection management with keep-alive
- Device management and state tracking
- Pairing service with verification
- Payload transfer (file sharing)
- Plugin manager and plugin implementations
- Desktop-specific features (clipboard, MPRIS, etc.)

**Dependencies:**
```toml
cosmic-ext-connect-core = { path = "../cosmic-ext-connect-core" }  # Shared TLS layer
```

**Uses from core:**
- TLS encryption layer (TlsConnection, TlsServer)
- Certificate management (CertificateInfo)
- Core packet types and error handling

### 3. Desktop Daemon (`cosmic-ext-connect-daemon`)

**Purpose:** Background service managing device connections

**Features:**
- Runs as systemd service
- mDNS device discovery
- Automatic device pairing
- Plugin execution (battery sync, clipboard, notifications)
- DBus interface for IPC with applet
- MPRIS media player control
- COSMIC notifications integration

**Dependencies:**
```toml
cosmic-ext-connect-protocol = { workspace = true }
```

### 4. COSMIC Applet (`cosmic-ext-applet-connect`)

**Purpose:** User interface in COSMIC panel

**Features:**
- Device status display
- Quick actions (ping, share, find phone)
- Pairing UI
- Battery level display
- Connection management

**Dependencies:**
```toml
libcosmic = { git = "https://github.com/pop-os/libcosmic" }
cosmic-ext-connect-protocol = { workspace = true }
```

### 5. Android App (`cosmic-connect-android`)

**Location:** https://github.com/olafkfreund/cosmic-connect-android

**Purpose:** Android client using shared Rust core

**Architecture:**
- **Rust Core:** Uses `cosmic-ext-connect-core` via FFI
- **Kotlin Layer:** Android UI, system integration, Material 3 design
- **Bridge:** uniffi-rs generates Kotlin bindings

**Features:**
- Jetpack Compose UI
- MVVM architecture
- Kotlin coroutines
- Material 3 design
- Targets Android 14+

## Integration Points

### Desktop ← Core Integration

The desktop app uses cosmic-ext-connect-core for the TLS layer:

```rust
// cosmic-ext-connect-protocol/src/lib.rs
pub use cosmic_connect_core::crypto::{
    CertificateInfo,
    DeviceInfo as TlsDeviceInfo,
    TlsConfig,
    TlsConnection,
    TlsServer,
    should_initiate_connection,
};
pub use cosmic_connect_core::{Packet as CorePacket, ProtocolError as CoreProtocolError};
```

### Android ← Core Integration

The Android app uses cosmic-ext-connect-core via Kotlin FFI:

```kotlin
// Generated Kotlin bindings from uniffi-rs
import uniffi.cosmic_connect_core.*

val packet = createPacket("kdeconnect.ping", emptyMap())
val discovery = startDiscovery(callback)
val pluginManager = createPluginManager()
```

## Protocol Version Compatibility

- **cosmic-ext-connect-core:** Protocol v7 (stable)
- **cosmic-ext-connect-protocol:** Protocol v8 (desktop extensions)
- **Android app:** Protocol v7 (via core)

All implementations maintain backward compatibility with KDE Connect ecosystem.

## Development Workflow

### For Desktop Development

```bash
# 1. Ensure cosmic-ext-connect-core is cloned as sibling directory
cd ~/Source/GitHub/
git clone https://github.com/olafkfreund/cosmic-ext-connect-core

# 2. Work on desktop app
cd cosmic-connect-desktop-app
nix develop
cargo build
```

### For Android Development

```bash
# 1. Clone Android app
git clone https://github.com/olafkfreund/cosmic-connect-android

# 2. cosmic-ext-connect-core is included as submodule or dependency
cd cosmic-connect-android
# Build generates Kotlin bindings automatically
./gradlew build
```

### For Core Library Development

```bash
# 1. Work on shared core
cd cosmic-ext-connect-core
cargo build

# 2. Test in desktop app
cd ../cosmic-connect-desktop-app
cargo build

# 3. Test in Android app
cd ../cosmic-connect-android
# Rebuild to get updated bindings
./gradlew build
```

## Plugin System

Plugins are defined in the shared core and implemented platform-specifically:

**Supported Plugins:**
- ✅ Battery - Battery status sync
- ✅ Clipboard - Clipboard sharing
- ✅ Connectivity Report - Network status
- ✅ Contacts - Contact sync
- ✅ Find My Phone - Ring device remotely
- ✅ MPRIS - Media player control
- ✅ Notification - Notification sync
- ✅ Ping - Connection testing
- ✅ Presenter - Remote presentation control
- ✅ Remote Input - Mouse/keyboard control
- ✅ Run Command - Execute commands
- ✅ Share - File/link sharing
- ✅ Telephony - Call/SMS notifications

Each plugin implements the `Plugin` trait defined in cosmic-ext-connect-core.

## Security

- **TLS 1.2+:** All connections encrypted using rustls
- **Certificate Pinning:** Devices verify each other's certificates
- **Pairing Verification:** User must approve new devices
- **No OpenSSL:** Avoids common OpenSSL vulnerabilities
- **Rust Memory Safety:** Core logic written in memory-safe Rust

## Future Improvements

1. **Enhanced Core Library:**
   - Add more plugins to shared core
   - Improve FFI bindings generation
   - Protocol v8 support in core

2. **Desktop Features:**
   - Additional desktop integrations
   - Better COSMIC Desktop integration
   - Enhanced file transfer UI

3. **Android Features:**
   - iOS app using same core
   - More Material 3 components
   - Enhanced notification handling

## Dependency Management

### Current Setup (Development)

The desktop app currently uses a **local path dependency** for cosmic-ext-connect-core:

```toml
# Cargo.toml
[workspace.dependencies]
cosmic-ext-connect-core = { path = "../cosmic-ext-connect-core" }
```

**Advantages:**
- ✅ Immediate testing of core changes in desktop app
- ✅ No need to push/tag/update versions during development
- ✅ Easier debugging across module boundaries
- ✅ Works well for active development workflow

### Alternative: Git Dependency (Production/CI)

For production builds or CI environments, you can use a git dependency:

```toml
# Cargo.toml
[workspace.dependencies]
# Use main branch (latest)
cosmic-ext-connect-core = { git = "https://github.com/olafkfreund/cosmic-ext-connect-core", branch = "main" }

# Or use a specific tag (stable)
cosmic-ext-connect-core = { git = "https://github.com/olafkfreund/cosmic-ext-connect-core", tag = "v0.1.2-alpha" }
```

**Advantages:**
- ✅ No need for local clone
- ✅ Easier for CI/CD pipelines
- ✅ Version pinning with tags
- ✅ Automatic dependency resolution

**Recommendation:** Keep local path for development, use git dependency for releases.

## Verification Checklist

### ✅ Confirmed Working

1. **cosmic-ext-connect-core Integration:**
   - ✅ Repository: https://github.com/olafkfreund/cosmic-ext-connect-core
   - ✅ Local path correctly configured: `../cosmic-ext-connect-core`
   - ✅ TLS/crypto layer exports verified
   - ✅ Latest commit: `db60b5b` - TLS transport layer extracted

2. **Desktop App Integration:**
   - ✅ Imports from cosmic_connect_core::crypto working
   - ✅ CertificateInfo, TlsConnection, TlsServer available
   - ✅ Project builds successfully in Nix environment
   - ✅ cosmic-ext-connect-protocol correctly separated from core

3. **Android App Compatibility:**
   - ✅ Repository: https://github.com/olafkfreund/cosmic-connect-android
   - ✅ Uses uniffi-rs for Kotlin FFI bindings
   - ✅ Targets 70%+ code sharing with desktop
   - ✅ Protocol v7 compatibility maintained

4. **Plugin System:**
   - ✅ Shared plugin definitions in cosmic-ext-connect-core
   - ✅ Desktop implementations in cosmic-ext-connect-protocol
   - ✅ Android implementations via FFI will use same core

## References

- [cosmic-ext-connect-core](https://github.com/olafkfreund/cosmic-ext-connect-core) - Shared Rust library
- [cosmic-connect-android](https://github.com/olafkfreund/cosmic-connect-android) - Android app
- [cosmic-connect-desktop-app](https://github.com/olafkfreund/cosmic-connect-desktop-app) - This repository
- [KDE Connect Protocol](https://community.kde.org/KDEConnect) - Protocol specification
- [uniffi-rs](https://github.com/mozilla/uniffi-rs) - FFI binding generator
- [rustls](https://github.com/rustls/rustls) - TLS implementation
