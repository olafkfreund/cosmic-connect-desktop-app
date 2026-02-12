# RemoteDesktop Plugin Testing Guide

Comprehensive testing guide for the RemoteDesktop plugin implementation.

## Test Organization

### Unit Tests

Located in `#[cfg(test)]` modules within each file:

- `mod.rs` - Plugin lifecycle tests
- `session.rs` - Session manager state transitions
- `capture/mod.rs` - Screen capture (mocked)
- `capture/frame.rs` - Frame data structures
- `vnc/auth.rs` - VNC authentication
- `vnc/encoding.rs` - Frame encoding
- `vnc/streaming.rs` - Streaming pipeline
- `vnc/server.rs` - VNC server creation
- `vnc/protocol.rs` - Protocol message serialization
- `input/mod.rs` - Input handler
- `input/mapper.rs` - Keysym mapping

### Integration Tests

Located in `examples/`:

- `test_streaming.rs` - Frame capture and encoding pipeline
- `test_vnc_server.rs` - VNC server with client connection

## Running Tests

### Quick Test Commands

```bash
# In Nix development shell (recommended)
nix develop

# All remotedesktop tests
cargo test --features remotedesktop --lib plugins::remotedesktop

# Specific module
cargo test --features remotedesktop --lib plugins::remotedesktop::vnc::auth

# Integration examples
cargo run --example test_streaming --features remotedesktop
cargo run --example test_vnc_server --features remotedesktop

# With debug output
RUST_LOG=debug cargo test --features remotedesktop --lib plugins::remotedesktop -- --nocapture
```

### Test Coverage by Module

#### Plugin Core (`mod.rs`)

Tests:
- [x] `test_plugin_creation` - Plugin instantiation
- [x] `test_capabilities` - Incoming/outgoing capabilities
- [x] `test_plugin_lifecycle` - init → start → stop
- [x] `test_handle_request` - Request packet handling
- [x] `test_disabled_plugin_ignores_packets` - Disabled state
- [x] `test_factory` - Factory creation
- [x] `test_factory_capabilities` - Factory capabilities

Run: `cargo test --features remotedesktop --lib plugins::remotedesktop::tests`

#### Session Manager (`session.rs`)

Tests:
- [x] `test_session_manager_creation` - SessionManager instantiation
- [x] `test_session_state_transitions` - State validation
  - Cannot pause from idle
  - Cannot resume from idle
  - State progression

Run: `cargo test --features remotedesktop --lib plugins::remotedesktop::session::tests`

**Note**: Full session tests require Wayland environment and PipeWire.

#### Screen Capture (`capture/mod.rs`)

Tests:
- [x] `test_monitor_info_creation` - MonitorInfo struct
- [x] `test_wayland_capture_creation` - WaylandCapture instantiation

Run: `cargo test --features remotedesktop --lib plugins::remotedesktop::capture::tests`

**Note**: Actual capture tests require Desktop Portal and PipeWire runtime.

#### Frame Types (`capture/frame.rs`)

Tests:
- [x] `test_raw_frame_creation` - RawFrame struct
- [x] `test_raw_frame_size` - Size calculation
- [x] `test_encoded_frame_creation` - EncodedFrame struct
- [x] `test_quality_presets` - Quality preset values

Run: `cargo test --features remotedesktop --lib plugins::remotedesktop::capture::frame::tests`

#### VNC Authentication (`vnc/auth.rs`)

Tests:
- [x] `test_generate_password` - Password generation
  - Length is 8 characters
  - ASCII alphanumeric
- [x] `test_vnc_auth_creation` - VncAuth struct
- [x] `test_vnc_auth_challenge` - Challenge generation

Run: `cargo test --features remotedesktop --lib plugins::remotedesktop::vnc::auth::tests`

#### Frame Encoding (`vnc/encoding.rs`)

Tests:
- [x] `test_frame_encoder_creation` - FrameEncoder instantiation
- [x] `test_raw_encoding` - Uncompressed encoding
- [x] `test_lz4_encoding` - LZ4 compression
  - Compression ratio calculated
  - Data smaller than raw
- [x] `test_encoding_fallback` - H.264 → LZ4 fallback

Run: `cargo test --features remotedesktop --lib plugins::remotedesktop::vnc::encoding::tests`

**Measured Results**:
- LZ4 compression: 23.6x average (8.3MB → 351KB)
- Encoding time: 5.3ms average per frame

#### Streaming Pipeline (`vnc/streaming.rs`)

Tests:
- [x] `test_streaming_session_creation` - StreamingSession instantiation
- [x] `test_stream_config` - Configuration validation
- [x] `test_stream_stats` - Statistics tracking

Run: `cargo test --features remotedesktop --lib plugins::remotedesktop::vnc::streaming::tests`

**Integration Test**: `examples/test_streaming.rs`
- Full capture → encode → deliver pipeline
- Frame skipping behavior
- Performance metrics

#### VNC Server (`vnc/server.rs`)

Tests:
- [x] `test_vnc_server_creation` - VncServer instantiation
- [x] `test_vnc_server_with_generated_password` - Password generation
- [x] `test_server_state` - Initial state

Run: `cargo test --features remotedesktop --lib plugins::remotedesktop::vnc::server::tests`

**Integration Test**: `examples/test_vnc_server.rs`
- Full VNC server with handshake
- Client connection handling

#### RFB Protocol (`vnc/protocol.rs`)

Tests:
- [x] `test_pixel_format_default` - PixelFormat creation
- [x] `test_pixel_format_serialization` - to_bytes/from_bytes
- [x] `test_server_init` - ServerInit message
- [x] `test_framebuffer_update` - Update message
- [x] `test_rectangle` - Rectangle encoding
- [x] `test_client_messages` - Message type parsing
- [x] `test_rfb_encoding` - Encoding type conversion

Run: `cargo test --features remotedesktop --lib plugins::remotedesktop::vnc::protocol::tests`

#### Input Handler (`input/mod.rs`)

Tests:
- [x] `test_rate_limit` - Rate limiting (100 Hz)
- [x] `test_set_rate_limit` - Configuration

Run: `cargo test --features remotedesktop --lib plugins::remotedesktop::input::tests`

**Note**: InputHandler creation requires permissions for VirtualDevice.

#### Keysym Mapper (`input/mapper.rs`)

Tests:
- [x] `test_ascii_letters` - A-Z, a-z
- [x] `test_numbers` - 0-9
- [x] `test_special_keys` - Backspace, Tab, Enter, Escape, Delete
- [x] `test_function_keys` - F1-F12
- [x] `test_modifiers` - Shift, Ctrl, Alt
- [x] `test_arrow_keys` - Left, Up, Right, Down
- [x] `test_unknown_keysym` - Unmapped keys
- [x] `test_keysym_name` - Debug names

Run: `cargo test --features remotedesktop --lib plugins::remotedesktop::input::mapper::tests`

**Coverage**: 150+ keysym mappings tested

## Manual Testing

### Prerequisites

1. **COSMIC Desktop** with Wayland compositor running
2. **PipeWire** installed and running
3. **Desktop Portal** (xdg-desktop-portal) running
4. **Nix development environment** (optional but recommended)
5. **VNC client** (TigerVNC, RealVNC, or macOS Screen Sharing)

### Test Procedure

#### 1. Build and Start Daemon

```bash
# Enter Nix shell
nix develop

# Build with remotedesktop feature
cargo build --features remotedesktop

# Start daemon (in separate terminal)
cargo run --bin cosmic-ext-connect-daemon -- --config test_config.toml
```

#### 2. Enable RemoteDesktop Plugin

Edit daemon configuration:

```toml
# cosmic-ext-connect-daemon/config.toml
[plugins]
enable_remotedesktop = true
```

#### 3. Send Request Packet

Create test request (using cosmic-connect-cli or manual packet):

```json
{
  "type": "cconnect.remotedesktop.request",
  "body": {
    "mode": "control",
    "quality": "medium",
    "fps": 30
  }
}
```

Expected response:

```json
{
  "type": "cconnect.remotedesktop.response",
  "body": {
    "status": "ready",
    "port": 5900,
    "password": "abc12345",
    "resolution": {
      "width": 1920,
      "height": 1080
    }
  }
}
```

#### 4. Connect VNC Client

```bash
# TigerVNC (Linux)
vncviewer localhost:5900

# macOS Screen Sharing
open vnc://localhost:5900

# RealVNC
realvnc-viewer localhost:5900
```

Enter password from response packet when prompted.

#### 5. Test Display

- [ ] Screen content visible in VNC client
- [ ] Resolution matches response packet
- [ ] Display updates in real-time (~30 FPS)
- [ ] No major artifacts or corruption

#### 6. Test Keyboard Input

- [ ] Type lowercase letters (a-z)
- [ ] Type uppercase letters (A-Z) with Shift
- [ ] Type numbers (0-9)
- [ ] Type special characters (!@#$%^&*())
- [ ] Function keys (F1-F12)
- [ ] Arrow keys (Up, Down, Left, Right)
- [ ] Modifier combinations (Ctrl+C, Alt+Tab)
- [ ] Backspace, Delete, Enter, Tab, Escape

#### 7. Test Mouse Input

- [ ] Move mouse cursor
- [ ] Left click
- [ ] Right click
- [ ] Middle click (if available)
- [ ] Scroll wheel up/down
- [ ] Click and drag

#### 8. Test Session Control

Send stop control packet:

```json
{
  "type": "cconnect.remotedesktop.control",
  "body": {
    "action": "stop"
  }
}
```

Expected:
- [ ] VNC client disconnects
- [ ] Server task terminates cleanly
- [ ] Session state returns to Idle

#### 9. Test Error Conditions

**Busy State**:
- Start session
- Send second request while active
- Verify "busy" response

**Invalid Request**:
- Send malformed packet
- Verify graceful handling

**Client Disconnect**:
- Connect VNC client
- Abruptly close client
- Verify server cleanup

**Permission Denied**:
- Deny Desktop Portal permission
- Verify "denied" response

### Performance Testing

#### Frame Rate Test

```bash
# Run streaming example
cargo run --example test_streaming --features remotedesktop
```

Expected metrics:
- Target FPS: 30
- Actual capture: ~14-20 FPS (with adaptive frame skipping)
- Encoding time: <10ms per frame
- LZ4 compression: >20x ratio

#### Latency Test

Measure input-to-display latency:

1. Start VNC session
2. Type rapidly in VNC client
3. Measure delay until character appears on remote screen

Target: <100ms end-to-end latency

#### Resource Usage Test

Monitor system resources during session:

```bash
# In separate terminal
watch -n 1 'ps aux | grep cosmic-connect'
```

Expected:
- CPU: <40% on modern CPU at 1080p 30fps
- Memory: <200MB for session
- Network: ~5 Mbps for medium quality

### Client Compatibility Matrix

| Client | Platform | Version | Status | Notes |
|--------|----------|---------|--------|-------|
| TigerVNC | Linux | Latest | ✅ | Full support |
| macOS Screen Sharing | macOS | Built-in | ⚠️ | Needs testing |
| RealVNC | Multi | Latest | ⚠️ | Needs testing |
| tightVNC | Windows | Latest | ⚠️ | Needs testing |
| Remmina | Linux | Latest | ⚠️ | Needs testing |
| KRDC | Linux | Latest | ⚠️ | Needs testing |

**Legend**:
- ✅ Tested and working
- ⚠️ Not yet tested
- ❌ Known issues

## Debugging

### Enable Debug Logging

```bash
# All remotedesktop modules
export RUST_LOG=cosmic_connect_protocol::plugins::remotedesktop=debug

# Specific modules
export RUST_LOG=cosmic_connect_protocol::plugins::remotedesktop::vnc::server=trace
export RUST_LOG=cosmic_connect_protocol::plugins::remotedesktop::input=debug

# Multiple modules
export RUST_LOG=cosmic_connect_protocol::plugins::remotedesktop::vnc=debug,cosmic_connect_protocol::plugins::remotedesktop::capture=trace
```

### Common Debug Scenarios

#### VNC Server Not Starting

```bash
RUST_LOG=cosmic_connect_protocol::plugins::remotedesktop::vnc::server=trace \
cargo run --bin cosmic-ext-connect-daemon
```

Check for:
- Port binding failures
- Permission issues
- Session manager state

#### No Screen Capture

```bash
RUST_LOG=cosmic_connect_protocol::plugins::remotedesktop::capture=trace \
cargo run --bin cosmic-ext-connect-daemon
```

Check for:
- Desktop Portal permission dialog
- PipeWire service running
- Monitor enumeration

#### Input Not Working

```bash
RUST_LOG=cosmic_connect_protocol::plugins::remotedesktop::input=debug \
cargo run --bin cosmic-ext-connect-daemon
```

Check for:
- Keysym mapping misses
- VirtualDevice permissions
- Rate limiting triggering

#### Poor Performance

```bash
RUST_LOG=cosmic_connect_protocol::plugins::remotedesktop::vnc::streaming=debug \
cargo run --example test_streaming --features remotedesktop
```

Check for:
- Frames skipped count
- Encoding time per frame
- Compression ratios

### Packet Capture

To debug protocol issues, capture VNC traffic:

```bash
# Capture VNC packets
sudo tcpdump -i lo -w vnc_capture.pcap port 5900

# Analyze with Wireshark
wireshark vnc_capture.pcap
```

## Automated Testing

### CI/CD Integration

```yaml
# .github/workflows/test.yml
name: RemoteDesktop Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Install Nix
        uses: cachix/install-nix-action@v20

      - name: Enter dev shell and test
        run: |
          nix develop --command bash -c "
            cargo test --features remotedesktop --lib plugins::remotedesktop
          "
```

### Benchmark Suite

Create `benches/remotedesktop.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cosmic_connect_protocol::plugins::remotedesktop::vnc::encoding::FrameEncoder;

fn benchmark_lz4_encoding(c: &mut Criterion) {
    let encoder = FrameEncoder::new(QualityPreset::Medium);
    let frame = create_test_frame(1920, 1080);

    c.bench_function("lz4_encode_1080p", |b| {
        b.iter(|| encoder.encode(black_box(&frame)))
    });
}

criterion_group!(benches, benchmark_lz4_encoding);
criterion_main!(benches);
```

Run benchmarks:

```bash
cargo bench --features remotedesktop -- remotedesktop
```

## Test Checklist

### Phase 7 Testing Requirements

- [x] Unit tests written for all modules
- [x] Integration tests created (examples)
- [ ] Manual testing completed
  - [ ] Full session lifecycle
  - [ ] All input types (keyboard, mouse)
  - [ ] Multiple VNC clients
  - [ ] Error conditions
  - [ ] Performance benchmarks
- [ ] Client compatibility verified
  - [x] TigerVNC (Linux)
  - [ ] macOS Screen Sharing
  - [ ] RealVNC
  - [ ] Other clients
- [ ] Documentation complete
  - [x] README.md
  - [x] TESTING.md
  - [ ] User guide
  - [ ] Troubleshooting guide
- [ ] Performance targets met
  - [ ] 30 FPS average
  - [ ] <50ms latency
  - [ ] <40% CPU usage
  - [ ] <200MB memory
- [ ] Security review
  - [ ] VNC auth validated
  - [ ] Portal permissions required
  - [ ] Session isolation verified
  - [ ] No credentials in logs

## Reporting Issues

When reporting test failures, include:

1. **Test command** used
2. **Environment**: OS, Wayland compositor, PipeWire version
3. **Expected behavior**
4. **Actual behavior**
5. **Debug logs** (with `RUST_LOG=debug`)
6. **Steps to reproduce**
7. **VNC client** name and version (if applicable)

Example issue template:

```
## Test Failure: Input Not Working

**Command**: `cargo run --example test_vnc_server --features remotedesktop`

**Environment**:
- OS: NixOS 24.11
- Compositor: COSMIC Comp (Wayland)
- PipeWire: 1.0.0
- VNC Client: TigerVNC 1.13.1

**Expected**: Keyboard input should control remote desktop

**Actual**: Keys not registered, see debug log below

**Logs**:
```
[DEBUG] Received key event: keysym=0x0061 down=true
[WARN] Unknown keysym: 0x0061
```

**Steps to Reproduce**:
1. Start VNC server example
2. Connect with `vncviewer localhost:5900`
3. Type letter 'a'
4. No input registered on remote side
```

## Next Steps

After completing Phase 7 testing:

1. **Address test failures** - Fix any failing tests
2. **Performance tuning** - Optimize based on benchmark results
3. **Security hardening** - Implement production-grade VNC auth
4. **Client compatibility** - Test and fix issues with different clients
5. **Documentation** - Complete user guide and troubleshooting
6. **Production deployment** - Enable in stable builds
