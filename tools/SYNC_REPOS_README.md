# Multi-Repository Sync System for COSMIC Connect

## Overview

This system helps you maintain consistency across three COSMIC Connect repositories:

1. **cosmic-connect-desktop-app** (Rust) - Desktop client [SOURCE]
2. **cosmic-ext-connect-core** (Rust) - Shared protocol library [TARGET]
3. **cosmic-connect-android** (Kotlin) - Android client [TARGET]

**Philosophy:** AI-assisted, human-verified synchronization. Claude analyzes changes, generates appropriate code, but YOU approve every sync.

---

## Best Approach: Interactive Sync with Claude

### Why This Approach?

❌ **Not Recommended:**
- Automated file syncing (error-prone, language-specific)
- Manual diffing (time-consuming, easy to miss changes)
- Copy-paste (loses language idioms, no verification)

✅ **Recommended:**
- Claude analyzes changes intelligently
- Claude generates language-appropriate code (Rust → Kotlin)
- You review and approve each change
- Claude creates commits with proper messages
- Full audit trail and rollback capability

---

## Setup

### 1. Repository Structure

Ensure all three repos are cloned locally:

```
~/Source/GitHub/
├── cosmic-connect-desktop-app/   # You're here
├── cosmic-ext-connect-core/           # Clone this
└── cosmic-connect-android/        # Clone this
```

Clone the other repos if needed:

```bash
cd ~/Source/GitHub/
git clone https://github.com/olafkfreund/cosmic-ext-connect-core
git clone https://github.com/olafkfreund/cosmic-connect-android
```

### 2. Update Configuration

Edit `.sync-config.yaml` and verify paths are correct:

```yaml
repos:
  core:
    path: /home/olafkfreund/Source/GitHub/cosmic-ext-connect-core
  android:
    path: /home/olafkfreund/Source/GitHub/cosmic-connect-android
```

### 3. Ready to Use!

No installation needed - just call Claude with `/sync-repos` when you need it.

---

## Usage Workflow

### When to Run Sync

Run `/sync-repos` when you:
- ✅ Change protocol version
- ✅ Add/modify packet types
- ✅ Change error types
- ✅ Update constants (ports, timeouts)
- ✅ Add new capabilities
- ✅ Modify shared protocol logic
- ❌ Change desktop-only code (no sync needed)
- ❌ Update UI/daemon code (no sync needed)

### Basic Workflow

```bash
# In cosmic-connect-desktop-app directory
1. Make your changes to protocol code
2. Commit changes locally
3. Call Claude: "/sync-repos"
4. Review Claude's analysis
5. Approve/reject each sync
6. Claude generates code for each repo
7. Review generated code
8. Claude creates commits in target repos
9. Push all repos
```

### Example Session

```
You: I just updated the protocol version from 7 to 8 and added a new
     telephony packet type. Can you sync these changes?

Claude: Let me analyze the changes...

🔍 Analysis Complete

I found 2 changes that need syncing:

1. **Protocol Version: 7 → 8** [CRITICAL]
   - File: cosmic-ext-connect-protocol/src/lib.rs
   - Line: pub const PROTOCOL_VERSION: u32 = 8;
   - Sync to: core, android

2. **New Packet Type: kdeconnect.telephony.request_mute**
   - File: cosmic-ext-connect-protocol/src/plugins/telephony.rs
   - Sync to: android

Let's handle these one by one.

---

Change 1: Protocol Version Update

This affects both repositories. Here's what I'll do:

🦀 cosmic-ext-connect-core:
```rust
pub const PROTOCOL_VERSION: u32 = 8;
```

🤖 cosmic-connect-android:
```kotlin
object ProtocolConstants {
    const val PROTOCOL_VERSION = 8
}
```

Approve this sync? [y/n]

You: y

Claude: ✅ Synced protocol version

Commits created:
- core: feat: bump protocol version to 8
- android: feat: bump protocol version to 8

---

Change 2: New Telephony Packet

This requires Android implementation. I'll generate the packet data class:

🤖 cosmic-connect-android:

```kotlin
// In app/src/main/java/org/kde/kdeconnect/Plugins/TelephonyPlugin/

data class TelephonyMuteRequest(
    val muted: Boolean
) : NetworkPacket(NetworkPacket.PACKET_TYPE_TELEPHONY_REQUEST_MUTE) {

    companion object {
        const val PACKET_TYPE_TELEPHONY_REQUEST_MUTE = "kdeconnect.telephony.request_mute"
    }

    fun toNetworkPacket(): NetworkPacket {
        return NetworkPacket(PACKET_TYPE_TELEPHONY_REQUEST_MUTE).apply {
            set("muted", muted)
        }
    }
}
```

And add the handler in TelephonyPlugin.kt:

```kotlin
override fun onPacketReceived(np: NetworkPacket): Boolean {
    when (np.type) {
        // ... existing handlers ...

        TelephonyMuteRequest.PACKET_TYPE_TELEPHONY_REQUEST_MUTE -> {
            val muted = np.getBoolean("muted")
            // TODO: Implement mute functionality
            handleMuteRequest(muted)
            return true
        }
    }
    return false
}
```

Approve this sync? [y/n]

You: y

Claude: ✅ Synced telephony packet type

Created commit in android:
- feat(telephony): add request_mute packet support

---

Summary:
✅ 2 changes synced successfully
📝 Commits created in 2 repositories
🧪 Recommended: Run tests in both core and android

Next steps:
1. Review commits in each repo
2. Run `cargo test` in core
3. Build android app and test telephony feature
4. Push all repositories

All done! 🎉
```

---

## Advanced Features

### Handling Complex Changes

**Multi-Step Changes:**
```
You: I refactored the entire error system. Can you help sync this?

Claude: This is a complex change affecting multiple components.
        Let me break it down into steps:

        Step 1: Update error enum in core
        Step 2: Update Android error sealed class
        Step 3: Update error messages in Android strings.xml
        Step 4: Update error handling in Android NetworkPacket

        Shall we proceed step-by-step? [y/n]
```

**Conflict Detection:**
```
Claude: ⚠️ Warning: Android repo has uncommitted changes to ProtocolConstants.kt

        Your change: PROTOCOL_VERSION = 8
        Android's WIP: PROTOCOL_VERSION = 7 (modified 3 hours ago)

        Options:
        a) Stash android changes and apply sync
        b) Skip this sync and review android changes first
        c) Merge both changes manually

        What would you like to do? [a/b/c]
```

**Breaking Changes:**
```
Claude: 🚨 Breaking Change Detected

        Removing packet type "kdeconnect.battery" would break Android compatibility.

        Impact:
        - Android's BatteryPlugin depends on this
        - ~500 lines of Android code affected

        Recommendations:
        1. Keep packet type for backward compatibility
        2. Mark as deprecated in both repos
        3. Add migration guide to CHANGELOG.md

        Shall I:
        a) Keep packet type and mark deprecated
        b) Remove anyway (breaking change)
        c) Cancel this sync

        Choose [a/b/c]:
```

---

## Language Translation Guide

### Rust → Kotlin Mapping

| Rust | Kotlin | Example |
|------|--------|---------|
| `pub enum` | `sealed class` | Error types |
| `pub struct` | `data class` | Packet data |
| `pub const` | `const val` | Constants |
| `Option<T>` | `T?` | Nullable types |
| `Result<T, E>` | `Result<T>` or exceptions | Error handling |
| `Vec<T>` | `List<T>` | Collections |
| `HashMap<K, V>` | `Map<K, V>` | Maps |
| `u64` | `Long` | Numbers |
| `String` | `String` | Strings |

### Example Translations

**Enums:**
```rust
// Rust
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Failed,
}
```

```kotlin
// Kotlin (generated by Claude)
sealed class ConnectionState {
    object Disconnected : ConnectionState()
    object Connecting : ConnectionState()
    object Connected : ConnectionState()
    object Failed : ConnectionState()
}
```

**Structs:**
```rust
// Rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub device_type: String,
    pub protocol_version: u32,
}
```

```kotlin
// Kotlin (generated by Claude)
@Serializable
data class DeviceInfo(
    val deviceId: String,
    val deviceName: String,
    val deviceType: String,
    val protocolVersion: Int
)
```

---

## Configuration Reference

### .sync-config.yaml Structure

```yaml
# Repositories
repos:
  desktop-app:
    role: source  # Primary source
  core:
    role: target  # Receives protocol changes
  android:
    role: target  # Receives protocol + UI changes

# What to track
sync_components:
  protocol:
    files: [protocol definitions]
    sync_to: [core, android]

  error_types:
    files: [error.rs]
    sync_to: [core, android]

# Triggers (what changes need attention)
triggers:
  protocol_version_change:
    severity: critical
    sync_to: [core, android]

  new_packet_type:
    severity: high
    sync_to: [android]

# Exclusions (what NOT to sync)
exclude:
  - "cosmic-ext-connect-daemon/**"  # Desktop-only
  - "**/tests/**"               # Test code
```

### Adding Custom Triggers

Edit `.sync-config.yaml`:

```yaml
triggers:
  custom_capability:
    files: ["cosmic-ext-connect-protocol/src/plugins/custom/*.rs"]
    pattern: "pub const CAPABILITY"
    sync_to: [android]
    severity: medium
```

---

## Best Practices

### 1. Sync Early and Often
✅ Sync after each protocol change
❌ Don't accumulate many changes before syncing

### 2. Review Every Change
✅ Read Claude's generated code carefully
✅ Ask Claude to explain if unclear
❌ Don't blindly approve all syncs

### 3. Test Both Sides
✅ Run `cargo test` in core after sync
✅ Build Android app and test features
❌ Don't assume generated code works

### 4. Version Appropriately
✅ Bump patch version for minor syncs
✅ Bump minor version for new features
✅ Bump major version for breaking changes
❌ Don't forget to update version numbers

### 5. Document Changes
✅ Update CHANGELOG.md in each repo
✅ Add migration guides for breaking changes
❌ Don't leave future developers guessing

---

## Troubleshooting

### Q: Claude says "no changes detected" but I made changes
**A:** Ensure you committed your changes first. Claude analyzes committed changes, not working directory changes.

### Q: Claude generated incorrect Kotlin code
**A:** Tell Claude to fix it:
```
You: The Kotlin code should use StateFlow instead of LiveData
Claude: You're right! Let me regenerate with StateFlow...
```

### Q: I want to skip syncing a specific change
**A:** Just say "no" or "skip" when Claude asks for approval

### Q: How do I undo a sync?
**A:** Ask Claude:
```
You: Please rollback the last sync to cosmic-connect-android
Claude: I'll revert the commit and restore the previous state...
```

### Q: Claude missed a change
**A:** Point it out:
```
You: You missed syncing the new TIMEOUT constant
Claude: You're right, let me sync that now...
```

---

## Workflow Diagram

```
┌─────────────────────────────────────────┐
│ 1. Make changes in desktop-app          │
│    - Update protocol/error/constants    │
│    - Commit changes                     │
└────────────┬────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────┐
│ 2. Run /sync-repos                      │
│    - Claude analyzes changes            │
│    - Identifies sync targets            │
└────────────┬────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────┐
│ 3. Review & Approve                     │
│    - Claude presents each change        │
│    - You approve or reject              │
└────────────┬────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────┐
│ 4. Claude Generates Code                │
│    - Rust for core                      │
│    - Kotlin for android                 │
└────────────┬────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────┐
│ 5. Review Generated Code                │
│    - Check for correctness              │
│    - Request revisions if needed        │
└────────────┬────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────┐
│ 6. Claude Creates Commits               │
│    - One per repository                 │
│    - Proper commit messages             │
└────────────┬────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────┐
│ 7. Test & Push                          │
│    - Run tests in each repo             │
│    - Push to GitHub                     │
└─────────────────────────────────────────┘
```

---

## Next Steps

1. **Set up repositories** - Clone core and android if not already done
2. **Verify configuration** - Check paths in `.sync-config.yaml`
3. **Make a test change** - Update a constant and try `/sync-repos`
4. **Establish workflow** - Get comfortable with the interactive process

---

## Summary

This is **Claude-assisted pair programming for multi-repo maintenance**:

✅ **Intelligent**: Claude understands protocol changes
✅ **Interactive**: You approve every step
✅ **Language-aware**: Proper Rust ↔ Kotlin translation
✅ **Safe**: No automatic syncing, full audit trail
✅ **Flexible**: Handle simple syncs or complex refactors

**Remember:** Claude is your assistant, not an automation tool. Review, understand, and approve each change.

Happy syncing! 🚀
