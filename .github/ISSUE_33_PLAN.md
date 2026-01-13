# Issue #33 Implementation Plan: Plugin Packet Routing Enhancement

## Current State Assessment

### What's Already Implemented ✅
- Basic PluginManager with plugin registration
- Packet routing via `handle_packet()` method
- Capability aggregation and advertisement
- Plugin lifecycle management (init, start, stop)
- Config-based plugin enable/disable

### What Needs Enhancement 🔧

## Phase 1: Plugin Architecture Improvements

### 1.1 Per-Device Plugin Instances
**Goal**: Allow each device to have its own plugin instances with independent state

**Changes Needed**:
- Modify PluginManager to maintain per-device plugin instances
- Update plugin registration to support cloning/factory pattern
- Change packet routing to use device-specific plugin instances

**Files to Modify**:
- `kdeconnect-protocol/src/plugins/mod.rs` - PluginManager structure
- `kdeconnect-daemon/src/main.rs` - Initialization logic

### 1.2 Plugin State Management
**Goal**: Persist and restore plugin states per device

**Changes Needed**:
- Add state serialization/deserialization to Plugin trait
- Create plugin state storage directory structure
- Implement automatic state save on shutdown
- Load plugin states on device connection

**New Files**:
- Plugin state config format (JSON/TOML)
- State persistence layer

### 1.3 Enhanced Error Handling
**Goal**: Isolate plugin failures to prevent cascading issues

**Changes Needed**:
- Wrap each plugin call in error boundaries
- Add timeout handling for slow plugins
- Implement graceful degradation when plugins fail
- Add plugin health monitoring

**Implementation**:
- Timeout wrapper for async plugin operations
- Error recovery strategies
- Logging and metrics for plugin failures

## Phase 2: Capability Negotiation

### 2.1 Mutual Capability Matching
**Goal**: Ensure plugins only activate when both devices support them

**Changes Needed**:
- Store remote device capabilities
- Implement capability intersection logic
- Only enable plugins with mutual support
- Handle capability version mismatches

**Files to Modify**:
- `kdeconnect-protocol/src/device.rs` - Store remote capabilities
- `kdeconnect-protocol/src/plugins/mod.rs` - Capability matching logic

### 2.2 Version Negotiation
**Goal**: Handle different plugin protocol versions gracefully

**Changes Needed**:
- Track plugin version requirements
- Implement version compatibility checks
- Fallback to compatible version when needed

## Phase 3: Plugin Communication Enhancement

### 3.1 Packet Sending from Plugins
**Goal**: Allow plugins to send packets back to devices

**Current Status**: Need to verify this works
**Testing Needed**:
- Check if plugins can access device connection
- Verify packet sending through ConnectionManager
- Test bidirectional communication

### 3.2 Inter-Plugin Communication
**Goal**: Allow plugins to communicate with each other

**Future Enhancement** (optional for this issue):
- Plugin event bus
- Plugin message passing
- Shared state access

## Phase 4: Testing and Documentation

### 4.1 Integration Tests
- Per-device plugin instances
- Error isolation and recovery
- Capability negotiation
- Packet routing under load

### 4.2 Performance Tests
- Packet routing latency
- Memory usage with multiple devices
- Plugin initialization time

### 4.3 Documentation
- Plugin development guide
- State management patterns
- Error handling best practices

## Implementation Order

1. **Phase 1.1**: Per-Device Plugin Instances (HIGHEST PRIORITY)
   - This is the foundation for all other enhancements
   - Enables proper plugin state management

2. **Phase 1.3**: Error Handling (HIGH PRIORITY)
   - Critical for stability
   - Prevents one bad plugin from breaking everything

3. **Phase 2.1**: Capability Negotiation (MEDIUM PRIORITY)
   - Improves interoperability
   - Prevents protocol mismatches

4. **Phase 1.2**: State Persistence (MEDIUM PRIORITY)
   - Nice to have for UX
   - Can be added incrementally

5. **Phase 3.1**: Verify Plugin Communication (QUICK WIN)
   - May already work, just needs testing
   - Essential for functionality

6. **Phase 4**: Testing and Documentation (ONGOING)
   - Add tests as we implement each phase

## Success Criteria

- [ ] Each device has independent plugin instances
- [ ] Plugin failures don't crash the daemon
- [ ] Plugins only activate when both devices support them
- [ ] Plugin states persist across connections
- [ ] Comprehensive test coverage
- [ ] Performance within acceptable bounds

## Estimated Effort

- Phase 1.1: 2-3 days (complex architectural change)
- Phase 1.3: 1-2 days (error handling patterns)
- Phase 2.1: 1 day (capability logic)
- Phase 1.2: 1 day (state persistence)
- Phase 3.1: 0.5 days (verification and tests)
- Phase 4: Ongoing throughout

**Total**: ~1 week of focused work

## Open Questions

1. Should plugin instances be created per-connection or persist across reconnections?
2. What's the plugin state storage format (JSON, TOML, binary)?
3. Should we implement plugin hot-reload capability?
4. What timeout values are appropriate for plugin operations?
