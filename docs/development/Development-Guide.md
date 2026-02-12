# Development Guide

This guide provides detailed information for developers working on cosmic-applet-kdeconnect.

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Development Environment](#development-environment)
- [DBus Interface](#dbus-interface)
- [Plugin System](#plugin-system)
- [Payload Transfer System](#payload-transfer-system)
- [Configuration System](#configuration-system)
- [COSMIC Notifications](#cosmic-notifications)
- [Testing](#testing)
- [Common Tasks](#common-tasks)

## Architecture Overview

### Components

```
┌─────────────────────────────────────────────────────────────┐
│                     COSMIC Desktop                          │
│  ┌───────────────────────────┐  ┌─────────────────────────┐│
│  │   cosmic-applet-kdeconnect│  │  COSMIC Notifications   ││
│  │   (Panel Applet UI)       │  │  (freedesktop.org)      ││
│  └────────────┬──────────────┘  └──────────┬──────────────┘│
│               │ DBus                        │               │
│               │                             │               │
│  ┌────────────▼─────────────────────────────▼──────────────┐│
│  │           kdeconnect-daemon                             ││
│  │  ┌─────────────────────────────────────────────────┐   ││
│  │  │  Device Manager  │  Plugin Manager              │   ││
│  │  ├─────────────────────────────────────────────────┤   ││
│  │  │  Connection Manager  │  Pairing Service         │   ││
│  │  ├─────────────────────────────────────────────────┤   ││
│  │  │  Configuration       │  COSMIC Notifier         │   ││
│  │  └─────────────────────────────────────────────────┘   ││
│  └──────────────────────┬──────────────────────────────────┘│
│                         │ TCP/TLS                            │
└─────────────────────────┼────────────────────────────────────┘
                          │
                ┌─────────▼──────────┐
                │  Mobile Device     │
                │  (Android/iOS)     │
                │  KDE Connect App   │
                └────────────────────┘
```

### Data Flow

1. **Device Discovery**: UDP broadcast → Daemon receives → Updates DeviceManager
2. **Pairing**: UI → DBus → PairingService → TLS handshake → Certificate storage
3. **Connection**: TLS connection → ConnectionManager → Per-device plugin initialization
4. **Plugin Events**: Packet received → PluginManager routes → Plugin handles → Notifications/DBus signals
5. **Configuration**: UI → DBus → ConfigRegistry → JSON persistence

## Development Environment

### NixOS (Recommended)

The flake.nix provides a complete development environment with all dependencies:

```bash
# Enter development shell
nix develop

# Build everything
just build

# Run tests
just test
```

### Environment Variables

The development shell sets:
- `RUST_BACKTRACE=1` - Full error backtraces
- `RUST_LOG=debug` - Debug logging
- `PKG_CONFIG_PATH` - For finding system libraries
- `LD_LIBRARY_PATH` - For runtime libraries

### Build Issue: xkbcommon

If you encounter `xkbcommon` not found errors when building the applet:

```bash
# You must be in the nix development shell
nix develop

# Then build
cargo build --package cosmic-applet-kdeconnect
```

**Why?** The xkbcommon library is required by smithay-client-toolkit (Wayland support). It's included in the flake.nix but only available inside the nix-shell.

## DBus Interface

The daemon exposes a comprehensive DBus interface at:
- **Service**: `io.github.olafkfreund.CosmicExtConnect`
- **Path**: `/io/github/olafkfreund/CosmicExtConnect`
- **Interface**: `io.github.olafkfreund.CosmicExtConnect`

### Methods

#### Device Management

```rust
// List all known devices
list_devices() -> HashMap<String, DeviceInfo>

// Get specific device
get_device(device_id: String) -> DeviceInfo

// Get device connection state
get_device_state(device_id: String) -> String  // "connected", "paired", "reachable", "unknown"
```

#### Pairing

```rust
// Request pairing with a device
pair_device(device_id: String) -> Result<(), Error>

// Unpair from a device
unpair_device(device_id: String) -> Result<(), Error>
```

#### Plugin Actions

```rust
// Send a ping to a device
send_ping(device_id: String, message: Option<String>) -> Result<(), Error>

// Get battery status from a device
get_battery_status(device_id: String) -> BatteryStatus { level: i32, is_charging: bool }

// Send a notification to a device
send_notification(device_id: String, title: String, body: String) -> Result<(), Error>

// Share a file with a device
share_file(device_id: String, file_path: String) -> Result<(), Error>
```

#### Configuration

```rust
// Set device nickname
set_device_nickname(device_id: String, nickname: String) -> Result<(), Error>

// Enable/disable a plugin for a device
set_device_plugin_enabled(device_id: String, plugin_name: String, enabled: bool) -> Result<(), Error>

// Clear device-specific plugin override (use global config)
clear_device_plugin_override(device_id: String, plugin_name: String) -> Result<(), Error>

// Get device configuration as JSON
get_device_config(device_id: String) -> String  // JSON
```

### Signals

The daemon emits DBus signals for events:

```rust
// Device was discovered or added
signal DeviceAdded(device_id: String, device_info: DeviceInfo)

// Device was removed or forgotten
signal DeviceRemoved(device_id: String)

// Device connection state changed
signal DeviceStateChanged(device_id: String, state: String)

// Pairing request received from a device
signal PairingRequest(device_id: String)

// Pairing status changed
signal PairingStatusChanged(device_id: String, status: String)  // "paired", "rejected", "failed"

// Plugin event occurred
signal PluginEvent(device_id: String, plugin: String, data: String)  // JSON data
```

### Testing DBus Interface

Use `busctl` to interact with the daemon:

```bash
# List devices
busctl --user call io.github.olafkfreund.CosmicExtConnect \
  /io/github/olafkfreund/CosmicExtConnect \
  io.github.olafkfreund.CosmicExtConnect \
  ListDevices

# Send a ping
busctl --user call io.github.olafkfreund.CosmicExtConnect \
  /io/github/olafkfreund/CosmicExtConnect \
  io.github.olafkfreund.CosmicExtConnect \
  SendPing ss "device-id-here" "Hello!"

# Monitor signals
busctl --user monitor io.github.olafkfreund.CosmicExtConnect
```

## Plugin System

### Plugin Architecture

Plugins implement the `Plugin` trait:

```rust
#[async_trait]
pub trait Plugin: Send + Sync + Any {
    fn name(&self) -> &str;
    fn as_any(&self) -> &dyn Any;

    fn incoming_capabilities(&self) -> Vec<String>;
    fn outgoing_capabilities(&self) -> Vec<String>;

    async fn init(&mut self, device: &Device) -> Result<()>;
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;

    async fn handle_packet(&mut self, packet: &Packet, device: &mut Device) -> Result<()>;
}
```

### Plugin Lifecycle

1. **Registration**: Plugin factories registered with PluginManager
2. **Initialization**: When device connects, plugins initialized via `init()`
3. **Start**: Plugins auto-start after initialization
4. **Packet Handling**: Incoming packets routed to appropriate plugin
5. **Cleanup**: On disconnect, `stop()` called and plugin instance removed

### Plugin State Querying

To query plugin-specific state from DBus:

```rust
// Plugins implement as_any() for downcasting
let plugin: Box<dyn Plugin> = /* ... */;
let battery_plugin = plugin.as_any().downcast_ref::<BatteryPlugin>()?;
let status = battery_plugin.get_battery_status();
```

### Adding a New Plugin

1. Create plugin file in `kdeconnect-protocol/src/plugins/`
2. Implement `Plugin` trait
3. Create `PluginFactory` implementation
4. Register factory in daemon's `initialize_plugins()`
5. Add configuration options if needed
6. Add tests

Example:

```rust
// kdeconnect-protocol/src/plugins/example.rs
pub struct ExamplePlugin {
    device_id: Option<String>,
    state: Arc<RwLock<PluginState>>,
}

#[async_trait]
impl Plugin for ExamplePlugin {
    fn name(&self) -> &str { "example" }

    fn as_any(&self) -> &dyn Any { self }

    fn incoming_capabilities(&self) -> Vec<String> {
        vec!["kdeconnect.example".to_string()]
    }

    async fn handle_packet(&mut self, packet: &Packet, device: &mut Device) -> Result<()> {
        // Handle packet
        Ok(())
    }

    // ... other trait methods
}

pub struct ExamplePluginFactory;

impl PluginFactory for ExamplePluginFactory {
    fn name(&self) -> &str { "example" }
    fn create(&self) -> Box<dyn Plugin> { Box::new(ExamplePlugin::new()) }
    // ... capabilities
}
```

## Payload Transfer System

### Overview

The payload transfer system enables file transfers between devices via TCP connections.
When sharing a file, the sender creates a TCP server, sends a share packet with the port,
and the receiver connects to download the file.

### Architecture

```
Sender (Desktop)                                    Receiver (Mobile)
┌────────────────┐                                  ┌────────────────┐
│ ShareFile      │                                  │ Share Plugin   │
│ DBus Method    │                                  │                │
└───────┬────────┘                                  └────────────────┘
        │                                                     ▲
        │ 1. Extract metadata                                │
        ▼                                                     │
┌────────────────┐                                          │
│ FileTransferInfo│                                          │
└───────┬────────┘                                          │
        │                                                     │
        │ 2. Create PayloadServer                            │
        ▼                                                     │
┌────────────────┐                                          │
│ TCP Server     │                                          │
│ Port: 1739-1764│                                          │
└───────┬────────┘                                          │
        │                                                     │
        │ 3. Send share packet                               │
        ├────────────────────────────────────────────────────┤
        │                                                     │
        │ 4. Wait for connection                              │
        │◄────────────────────────────────────────────────────┤
        │                                                     │
        │ 5. Stream file data                                 │
        ├────────────────────────────────────────────────────►│
        │                                                     │
        │ 6. Close connection                                 │
        └─────────────────────────────────────────────────────┘
```

### Components

#### PayloadServer

TCP server for sending files:

```rust
use kdeconnect_protocol::{PayloadServer, FileTransferInfo};

// Extract file metadata
let file_info = FileTransferInfo::from_path("/path/to/file.pdf").await?;

// Create server on available port
let server = PayloadServer::new().await?;
let port = server.port(); // 1739-1764

// Send share packet with port info
// ... create and send packet ...

// Accept connection and transfer file
server.send_file("/path/to/file.pdf").await?;
```

**Features**:
- Automatic port selection (1739-1764)
- 64KB buffer for efficient streaming
- Connection timeout (30s)
- Transfer timeout (60s per operation)
- Progress logging

#### PayloadClient

TCP client for receiving files:

```rust
use kdeconnect_protocol::PayloadClient;

// Connect to sender's payload server
let client = PayloadClient::new("192.168.1.100", 1739).await?;

// Download file
client.receive_file("/tmp/received.pdf", 1048576).await?;
```

**Features**:
- Connection timeout handling
- Size validation
- Premature disconnection detection
- Progress tracking

#### FileTransferInfo

File metadata extraction:

```rust
use kdeconnect_protocol::FileTransferInfo;

let info = FileTransferInfo::from_path("/path/to/file").await?;
println!("Filename: {}", info.filename);
println!("Size: {} bytes", info.size);
println!("Created: {:?}", info.creation_time);
println!("Modified: {:?}", info.last_modified);

// Convert to SharePlugin's format
let share_info: FileShareInfo = info.into();
```

### DBus Integration

The daemon exposes file sharing via DBus:

```bash
# Share a file
busctl --user call io.github.olafkfreund.CosmicExtConnect \
  /io/github/olafkfreund/CosmicExtConnect \
  io.github.olafkfreund.CosmicExtConnect \
  ShareFile ss "device-id" "/path/to/file.pdf"
```

**Implementation Flow**:

1. **Validation**: Check device exists and is connected
2. **Metadata**: Extract file info using `FileTransferInfo::from_path()`
3. **Server Creation**: Start `PayloadServer` on available port
4. **Packet Creation**: Use `SharePlugin::create_file_packet()` with port
5. **Send Packet**: Send via `ConnectionManager::send_packet()`
6. **Background Transfer**: Spawn async task to handle file streaming
7. **Completion**: Log success or failure

### Protocol Details

#### Share Packet Format

```json
{
  "id": 1234567890,
  "type": "kdeconnect.share.request",
  "body": {
    "filename": "document.pdf",
    "creationTime": 1640000000000,
    "lastModified": 1640000000000,
    "open": false
  },
  "payloadSize": 1048576,
  "payloadTransferInfo": {
    "port": 1739
  }
}
```

#### Transfer Protocol

1. **Server Preparation**: Sender binds to port (1739-1764)
2. **Packet Exchange**: Sender sends share packet with port
3. **Connection**: Receiver connects to sender's IP:port
4. **Streaming**: Raw bytes transferred (64KB chunks)
5. **Completion**: Connection closed after `payloadSize` bytes
6. **Verification**: Receiver validates size matches

### Error Handling

All errors use the protocol's `ProtocolError` type:

```rust
// I/O errors (file not found, permission denied, network)
ProtocolError::Io(std::io::Error)

// Timeout errors
ProtocolError::Io(std::io::Error { kind: TimedOut, .. })

// Invalid input (no addresses resolved, invalid filename)
ProtocolError::InvalidPacket(String)
```

### Performance

- **Buffer Size**: 64KB for optimal streaming
- **Connection Timeout**: 30 seconds
- **Transfer Timeout**: 60 seconds per read/write
- **Throughput**: Network-limited, typically 10-100 MB/s on LAN

### Testing

```bash
# Test metadata extraction
cargo test test_file_transfer_info_from_path

# Test server creation
cargo test test_payload_server_creation

# Test full transfer
cargo test test_file_transfer_round_trip
```

### Common Issues

**Port Already in Use**:
```
Error: Failed to bind payload server - all ports in range 1739-1764 are in use
```
Solution: Close other KDE Connect instances or wait for ports to free

**Connection Timeout**:
```
Error: Connection timeout
```
Solution: Check firewall rules, ensure ports 1739-1764 open

**Size Mismatch**:
```
Error: Connection closed prematurely: received 512 bytes, expected 1024
```
Solution: Network interruption, retry transfer

## Configuration System

### Global Configuration

Located at `~/.config/kdeconnect/config.toml`:

```toml
[device]
name = "My COSMIC Desktop"
device_type = "desktop"
device_id = "uuid-here"

[network]
discovery_port = 1716

[plugins]
enable_ping = true
enable_battery = true
enable_notification = true
enable_share = true
enable_clipboard = true
enable_mpris = true
```

### Per-Device Configuration

Located at `~/.config/kdeconnect/device_configs.json`:

```json
{
  "device-id-here": {
    "device_id": "device-id-here",
    "nickname": "My Phone",
    "plugins": {
      "enable_ping": true,
      "enable_battery": null,  // null = use global config
      "enable_notification": false
    },
    "auto_accept_pairing": false,
    "auto_connect": true,
    "show_notifications": true
  }
}
```

### Configuration Priority

1. Per-device plugin override (if set)
2. Global plugin configuration
3. Default (true for all plugins)

### Configuration API

```rust
// Load configuration
let mut registry = DeviceConfigRegistry::new(&config_dir);
registry.load()?;

// Get or create device config
let config = registry.get_or_create("device-id");

// Set plugin override
config.set_plugin_enabled("battery", false);

// Check if plugin is enabled (respects override + global)
let enabled = config.is_plugin_enabled("battery", &global_config);

// Save changes
registry.save()?;
```

## COSMIC Notifications

### Notification System

The daemon integrates with COSMIC Desktop's notification system via freedesktop.org DBus:

```rust
// Create notifier
let notifier = CosmicNotifier::new().await?;

// Send a notification
notifier.send(
    NotificationBuilder::new("Title")
        .body("Notification body")
        .icon("phone-symbolic")
        .urgency(Urgency::Normal)
        .timeout(5000)
        .action("action-id", "Action Label")
).await?;
```

### Notification Types

The daemon sends notifications for:

1. **Pings**: When a device sends a ping
2. **Device Notifications**: Forwarded notifications from mobile device
3. **Pairing Requests**: With Accept/Reject action buttons
4. **File Received**: With Open/Show in Files actions
5. **Battery Low**: When device battery is low
6. **Device Connected/Disconnected**: Connection status changes

### Notification Helpers

```rust
// Ping notification
notifier.notify_ping("Device Name", Some("Message")).await?;

// Device notification (forwarded)
notifier.notify_from_device("Device Name", "App Name", "Title", "Text").await?;

// Pairing request (with actions)
notifier.notify_pairing_request("Device Name").await?;

// File received
notifier.notify_file_received("Device Name", "file.txt", "/path/to/file").await?;

// Battery low
notifier.notify_battery_low("Device Name", 15).await?;

// Connection status
notifier.notify_device_connected("Device Name").await?;
notifier.notify_device_disconnected("Device Name").await?;
```

### Notification Configuration

```rust
NotificationBuilder::new("Summary")
    .body("Body text")
    .icon("icon-name")
    .urgency(Urgency::Normal)  // Low, Normal, Critical
    .timeout(5000)  // milliseconds, 0 = no auto-dismiss
    .action("id", "Label")  // Add action button
    .hint("key", value)  // Custom hint
```

## Testing

### Test Organization

```
tests/
├── protocol_tests.rs       # Protocol library tests (91 tests)
└── plugin_integration_tests.rs  # Plugin integration tests (12 tests)
```

### Running Tests

```bash
# All tests
cargo test

# Specific package
cargo test -p kdeconnect-protocol
cargo test -p kdeconnect-daemon

# Specific test
cargo test test_battery_status_query

# With output
cargo test -- --nocapture

# Integration tests only
cargo test --test plugin_integration_tests
```

### Writing Plugin Tests

```rust
#[tokio::test]
async fn test_plugin_functionality() {
    // Create mock device
    let device = Device {
        info: DeviceInfo::new("Test Device", DeviceType::Phone, 1716),
        connection_state: ConnectionState::Connected,
        pairing_status: PairingStatus::Paired,
        // ...
    };

    // Create and initialize plugin
    let mut plugin = MyPlugin::new();
    plugin.init(&device).await.unwrap();

    // Create test packet
    let packet = Packet::new("kdeconnect.test", json!({ "key": "value" }));

    // Handle packet
    let mut device_mut = device;
    plugin.handle_packet(&packet, &mut device_mut).await.unwrap();

    // Verify state
    assert_eq!(plugin.get_state(), expected_state);
}
```

### Test Coverage

Current test coverage:
- Protocol library: ~90%
- Plugin implementations: ~85%
- Daemon integration: ~70%
- UI: Not yet tested (blocked by build)

## Common Tasks

### Adding a DBus Method

1. Add method to `KdeConnectInterface` in `kdeconnect-daemon/src/dbus.rs`:

```rust
#[interface(name = "io.github.olafkfreund.CosmicExtConnect")]
impl KdeConnectInterface {
    async fn my_new_method(&self, param: String) -> Result<String, zbus::fdo::Error> {
        // Implementation
        Ok("result".to_string())
    }
}
```

2. Test with busctl:

```bash
busctl --user call io.github.olafkfreund.CosmicExtConnect \
  /io/github/olafkfreund/CosmicExtConnect \
  io.github.olafkfreund.CosmicExtConnect \
  MyNewMethod s "parameter"
```

### Adding a Configuration Option

1. Add to `Config` struct in `kdeconnect-daemon/src/config.rs`
2. Add to default config in `Config::default()`
3. Add to `config.toml` template
4. Update documentation

### Debugging

```bash
# Run daemon with debug logs
RUST_LOG=debug cargo run --package kdeconnect-daemon

# Run with specific module logs
RUST_LOG=kdeconnect_daemon::plugins=trace cargo run --package kdeconnect-daemon

# Monitor DBus traffic
busctl --user monitor io.github.olafkfreund.CosmicExtConnect

# Check daemon status
systemctl --user status cosmic-kdeconnect-daemon

# View daemon logs
journalctl --user -u cosmic-kdeconnect-daemon -f
```

### Profiling

```bash
# Build with profiling
cargo build --release --package kdeconnect-daemon

# Run with perf
perf record -g ./target/release/kdeconnect-daemon
perf report

# Flamegraph
cargo install flamegraph
cargo flamegraph --package kdeconnect-daemon
```

## Best Practices

### Error Handling

- Use `anyhow::Result` for errors that cross boundaries
- Use `thiserror` for custom error types
- Always context errors: `.context("What failed")?`
- Log errors with tracing: `error!("Failed: {}", e)`

### Async/Await

- Prefer `tokio::spawn` for background tasks
- Use `Arc<RwLock<>>` for shared mutable state
- Always `.await` in non-blocking context
- Use channels for inter-task communication

### Logging

```rust
use tracing::{debug, info, warn, error};

debug!("Detailed information: {}", value);
info!("Important event occurred");
warn!("Something unexpected: {}", issue);
error!("Error occurred: {}", error);
```

### Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Use meaningful variable names
- Document public APIs
- Add tests for new functionality

## Resources

- [KDE Connect Protocol](https://invent.kde.org/network/kdeconnect-kde)
- [libcosmic Book](https://pop-os.github.io/libcosmic-book/)
- [zbus Documentation](https://docs.rs/zbus/)
- [tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Rust Async Book](https://rust-lang.github.io/async-book/)

## Getting Help

- Create an issue on GitHub
- Check existing documentation
- Review integration tests for examples
- Ask in COSMIC community chat
