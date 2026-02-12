# Automated Testing Guide

**Purpose:** Document automated testing procedures for COSMIC Connect development.

**Date:** 2026-01-16

---

## Overview

COSMIC Connect uses a comprehensive automated testing strategy with unit tests and integration tests to ensure code quality and plugin functionality.

### Test Categories

1. **Unit Tests** - Test individual components and functions in isolation
2. **Integration Tests** - Test plugin interactions and complete workflows
3. **Protocol Tests** - Verify KDE Connect protocol compliance

---

## Running Tests

### Run All Tests

```bash
# From project root
cargo test

# With output
cargo test -- --nocapture

# With verbose logging
RUST_LOG=debug cargo test -- --nocapture
```

### Run Specific Test Suite

```bash
# Protocol unit tests only
cargo test --package cosmic-ext-connect-protocol

# Daemon unit tests only
cargo test --package cosmic-ext-connect-daemon

# Integration tests only
cargo test --test plugin_integration_tests
```

### Run Individual Tests

```bash
# Run a specific test by name
cargo test test_battery_plugin_initialization

# Run all clipboard tests
cargo test clipboard

# Run all MPRIS tests
cargo test mpris
```

### Run Tests with Coverage

```bash
# Install tarpaulin if needed
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir target/coverage

# Open coverage report
xdg-open target/coverage/index.html
```

---

## Integration Test Suite

The integration test suite (cosmic-ext-connect-daemon/tests/plugin_integration_tests.rs) contains comprehensive end-to-end tests for all plugins.

### Test Structure

Tests are organized into categories:

1. **Basic Plugin Tests** (lines 30-127)
   - Plugin initialization
   - Capabilities verification
   - Packet creation

2. **Plugin Lifecycle Tests** (lines 142-237)
   - Plugin trait downcasting
   - Plugin manager operations
   - Multiple plugin coexistence

3. **End-to-End Integration Tests** (lines 240-628)
   - Complete packet exchange cycles
   - Multi-device scenarios
   - Plugin interaction workflows

### Key Integration Tests

#### Clipboard Synchronization
```bash
# Test bidirectional clipboard sync between mock devices
cargo test test_clipboard_sync_between_devices

# Test timestamp-based loop prevention
cargo test test_clipboard_timestamp_loop_prevention
```

**What it tests:**
- Device 1 sends clipboard content to Device 2
- Device 2 receives and updates clipboard
- Timestamps prevent sync loops
- Content integrity preserved

#### Share Plugin
```bash
# Test text sharing
cargo test test_share_plugin_text

# Test URL sharing
cargo test test_share_plugin_url

# Test file sharing
cargo test test_share_plugin_file
```

**What it tests:**
- Text share packet creation and handling
- URL validation and sharing
- File metadata in share packets

#### MPRIS Media Control
```bash
# Test MPRIS initialization
cargo test test_mpris_plugin_initialization

# Test media control commands
cargo test test_mpris_control_commands

# Test player list
cargo test test_mpris_player_list
```

**What it tests:**
- Play/pause, next, previous, stop commands
- Player list packet creation
- Command packet structure

#### Battery Status
```bash
# Test complete request/response cycle
cargo test test_battery_request_response_cycle
```

**What it tests:**
- Battery request packet creation
- Battery status response handling
- Status query from plugin

#### Notification System
```bash
# Test notification send and dismiss
cargo test test_notification_send_and_dismiss
```

**What it tests:**
- Notification packet creation
- Incoming notification handling
- Notification dismiss packets

#### Ping Exchange
```bash
# Test complete ping cycle
cargo test test_complete_ping_exchange
```

**What it tests:**
- Ping packet with message
- Bidirectional ping handling
- Packet structure validation

#### Multi-Device Management
```bash
# Test plugin manager with multiple devices
cargo test test_plugin_manager_multi_device
```

**What it tests:**
- Multiple devices with separate plugin instances
- Plugin initialization per device
- Device cleanup and plugin removal

#### Packet Routing
```bash
# Test routing packets to correct plugins
cargo test test_packet_routing_to_correct_plugin
```

**What it tests:**
- Different packet types route to correct plugins
- Plugin manager dispatches packets correctly
- No packet type conflicts

---

## Protocol Compliance Tests

Protocol tests verify KDE Connect specification compliance.

### Packet Structure Tests

```bash
# Test all packet creation functions
cargo test --package cosmic-ext-connect-protocol packet
```

**Verifies:**
- Correct packet type strings
- Required body fields present
- JSON serialization/deserialization
- Protocol version compatibility

### Capability Tests

```bash
# Test plugin capabilities
cargo test capabilities
```

**Verifies:**
- Incoming capabilities match supported packet types
- Outgoing capabilities declared correctly
- Capability negotiation compatibility

---

## Writing New Tests

### Unit Test Template

```rust
#[tokio::test]
async fn test_my_plugin_feature() -> Result<()> {
    // Setup
    let mut plugin = MyPlugin::new();
    let device = create_mock_device();
    plugin.init(&device).await?;

    // Execute
    let packet = plugin.create_some_packet("test_data");

    // Verify
    assert_eq!(packet.packet_type, "kdeconnect.expected.type");
    assert_eq!(packet.body["field"], "expected_value");

    Ok(())
}
```

### Integration Test Template

```rust
#[tokio::test]
async fn test_complete_workflow() -> Result<()> {
    // Setup two devices
    let mut plugin1 = Plugin::new();
    let mut plugin2 = Plugin::new();

    let device1 = create_mock_device();
    let mut device2 = create_mock_device();
    device2.info.device_id = "device2".to_string();

    plugin1.init(&device1).await?;
    plugin2.init(&device2).await?;

    // Device 1 sends to Device 2
    let packet = plugin1.create_packet("data");
    plugin2.handle_packet(&packet, &mut device2).await?;

    // Verify Device 2 state
    assert_eq!(plugin2.get_state(), expected_state);

    Ok(())
}
```

### Test Best Practices

1. **Use descriptive names** - `test_clipboard_sync_between_devices` not `test_clipboard_1`
2. **Test one thing** - Each test should verify a single behavior
3. **Include both success and error paths** - Test happy path and error handling
4. **Use mock devices** - Don't require real device connections
5. **Clean up resources** - Tests should not leave side effects
6. **Document complex tests** - Add comments explaining what's being tested

---

## Continuous Integration

### GitHub Actions Workflow

Tests run automatically on:
- Every push to main branch
- Every pull request
- Scheduled daily builds

### Local Pre-Commit Testing

```bash
# Run before committing
cargo test
cargo clippy -- -D warnings
cargo fmt -- --check

# Run full validation
./scripts/pre-commit-check.sh
```

---

## Test Coverage Goals

Current test coverage by component:

| Component | Coverage | Goal |
|-----------|----------|------|
| Protocol | 85% | 90% |
| Plugins | 80% | 90% |
| Daemon Core | 75% | 85% |
| Connection Manager | 70% | 85% |
| Plugin Manager | 90% | 95% |
| DBus Interface | 60% | 80% |

### Improving Coverage

1. **Identify gaps**
   ```bash
   cargo tarpaulin --out Html
   # Review coverage/index.html for untested code
   ```

2. **Add tests for uncovered code**
   - Focus on error paths
   - Edge cases
   - Complex conditionals

3. **Test new features**
   - Write tests before implementation (TDD)
   - Ensure new code has >80% coverage

---

## Debugging Test Failures

### View Detailed Output

```bash
# Show println! and logging output
cargo test -- --nocapture

# Show only failed tests
cargo test -- --nocapture --test-threads=1

# Run specific failing test with debug logging
RUST_LOG=debug cargo test test_name -- --nocapture
```

### Common Test Failures

#### "Device not found" Errors

**Cause:** Mock device not properly initialized
**Fix:**
```rust
let device = create_mock_device();
plugin.init(&device).await?;  // Must init before use
```

#### "Plugin not registered" Errors

**Cause:** Plugin factory not registered with PluginManager
**Fix:**
```rust
manager.register_factory(Arc::new(MyPluginFactory));
```

#### Timeout Errors

**Cause:** Async operations not completing
**Fix:**
```rust
use tokio::time::timeout;
use std::time::Duration;

timeout(Duration::from_secs(5), async_operation)
    .await
    .expect("Operation timed out");
```

#### Packet Structure Mismatches

**Cause:** Expected packet fields don't match actual
**Fix:**
```rust
// Print actual packet for debugging
println!("Packet: {:?}", packet);
println!("Body: {}", serde_json::to_string_pretty(&packet.body)?);
```

---

## Performance Testing

### Benchmarking

```bash
# Run benchmarks
cargo bench

# Benchmark specific component
cargo bench --package cosmic-ext-connect-protocol
```

### Latency Testing

```rust
use std::time::Instant;

#[tokio::test]
async fn test_packet_send_latency() {
    let start = Instant::now();

    // Send packet
    send_packet(&packet).await;

    let duration = start.elapsed();
    assert!(duration < Duration::from_millis(100),
           "Packet send took {:?}", duration);
}
```

---

## Test Data and Fixtures

### Mock Devices

```rust
/// Create standard test device (phone)
fn create_mock_device() -> Device {
    Device {
        info: DeviceInfo::new("Test Device", DeviceType::Phone, 1716),
        connection_state: ConnectionState::Connected,
        pairing_status: PairingStatus::Paired,
        is_trusted: true,
        // ...
    }
}

/// Create test device (desktop)
fn create_mock_desktop() -> Device {
    Device {
        info: DeviceInfo::new("Test Desktop", DeviceType::Desktop, 1716),
        // ...
    }
}
```

### Test Packets

```rust
/// Create test battery packet
fn create_test_battery_packet(charge: i32) -> Packet {
    let status = BatteryStatus {
        current_charge: charge,
        is_charging: false,
        threshold_event: 0,
    };
    BatteryPlugin::new().create_battery_packet(&status)
}

/// Create test ping packet
fn create_test_ping() -> Packet {
    PingPlugin::new().create_ping(Some("Test".to_string()))
}
```

---

## Test Maintenance

### Regular Tasks

1. **Weekly:**
   - Run full test suite: `cargo test`
   - Check for flaky tests
   - Review test output for warnings

2. **Monthly:**
   - Update test dependencies
   - Review test coverage reports
   - Add tests for new edge cases discovered

3. **Per Release:**
   - Run tests on multiple platforms
   - Verify all integration tests pass
   - Update test documentation

### Identifying Flaky Tests

```bash
# Run test 100 times to check for flakiness
for i in {1..100}; do
    cargo test test_name || echo "Failed on run $i"
done
```

---

## Related Documentation

- **Manual Testing:** docs/PLUGIN_TESTING_GUIDE.md - Manual testing with real devices
- **Debugging:** docs/DEBUGGING.md - Debug tools and techniques
- **Architecture:** docs/ARCHITECTURE.md - System design and plugin architecture
- **Contributing:** docs/CONTRIBUTING.md - Contribution guidelines

---

## Quick Reference

### Most Common Commands

```bash
# Run all tests
cargo test

# Run integration tests
cargo test --test plugin_integration_tests

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_clipboard_sync

# Check coverage
cargo tarpaulin --out Html

# Format and lint
cargo fmt && cargo clippy
```

### Test Files Locations

```
cosmic-ext-connect-daemon/
├── tests/
│   └── plugin_integration_tests.rs   # Integration test suite
└── src/
    ├── dbus.rs                       # Includes unit tests
    ├── connection/
    │   └── manager.rs                # Includes unit tests
    └── ...

cosmic-ext-connect-protocol/
└── src/
    └── plugins/
        ├── battery.rs                # Includes unit tests
        ├── clipboard.rs              # Includes unit tests
        ├── ping.rs                   # Includes unit tests
        └── ...                       # All plugins have tests
```

---

**Last Updated:** 2026-01-16
**Test Suite Status:** 43 integration tests, all passing
**Coverage:** ~80% overall, target 85%
