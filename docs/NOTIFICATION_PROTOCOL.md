# Notification Protocol Specification

> **Version:** 1.0.0
> **Last Updated:** 2026-02-02
> **Status:** Reference Documentation
> **Scope:** COSMIC Connect Ecosystem (Desktop, Android, Core)

---

## Table of Contents

1. [Overview](#overview)
2. [Packet Format](#packet-format)
3. [Field Mappings](#field-mappings)
4. [Rich Content](#rich-content)
5. [Security](#security)
6. [Implementation Checklist](#implementation-checklist)
7. [Reference Files](#reference-files)

---

## Overview

### Purpose

This document defines the notification handling protocol for the COSMIC Connect ecosystem. It serves as the authoritative reference for implementing notification synchronization between:

- **COSMIC Connect Desktop** (cosmic-ext-connect-daemon, cosmic-ext-applet-connect)
- **COSMIC Connect Android** (cosmic-connect-android)
- **COSMIC Connect Core** (cosmic-ext-connect-core FFI library)
- **COSMIC Notifications NG** (cosmic-notifications-ng daemon)

### Protocol Compatibility

The notification protocol is compatible with:

| Protocol | Version | Prefix | Notes |
|----------|---------|--------|-------|
| COSMIC Connect | 1.0+ | `cconnect.notification` | Primary protocol |
| KDE Connect | 7+ | `kdeconnect.notification` | Backward compatible |

Both prefixes are handled identically. Outgoing packets use `cconnect.notification`.

### Architecture Flow

```
+------------------+     +-------------------+     +--------------------+
|  Android Phone   |     |  COSMIC Connect   |     |  COSMIC Desktop    |
|                  |     |      Daemon       |     |   Notification     |
|  Notification    | --> |                   | --> |      Daemon        |
|  (Android OS)    |     |  Protocol Layer   |     |  (org.freedesktop) |
+------------------+     +-------------------+     +--------------------+
                                  |
                                  v
                         +-------------------+
                         | DBus Listener     |
                         | (Desktop -> Phone)|
                         +-------------------+
```

---

## Packet Format

### Packet Types

| Type | Direction | Purpose |
|------|-----------|---------|
| `cconnect.notification` | Bidirectional | Send/cancel notification |
| `cconnect.notification.request` | Bidirectional | Request all/dismiss one |
| `cconnect.notification.action` | Phone -> Desktop | Trigger action button |
| `cconnect.notification.reply` | Phone -> Desktop | Inline reply |

### Base Packet Structure

All packets follow this JSON structure with a newline terminator:

```json
{
  "id": 1234567890,
  "type": "cconnect.notification",
  "body": { ... }
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | `i64` | Yes | Unix timestamp in milliseconds |
| `type` | `string` | Yes | Packet type identifier |
| `body` | `object` | Yes | Packet-specific payload |

---

### Notification Packet

**Type:** `cconnect.notification`

#### Basic Notification (Backward Compatible)

```json
{
  "id": 1704067200000,
  "type": "cconnect.notification",
  "body": {
    "id": "notification-id-123",
    "appName": "Messages",
    "title": "New Message",
    "text": "Hello from your phone!",
    "ticker": "Messages: New Message - Hello from your phone!",
    "isClearable": true,
    "time": "1704067200000",
    "silent": "false"
  }
}
```

#### Extended Notification (Rich Content)

```json
{
  "id": 1704067200000,
  "type": "cconnect.notification",
  "body": {
    "id": "desktop-Thunderbird-1704067200000",
    "appName": "Thunderbird",
    "title": "New Email",
    "text": "You have a new message from Alice",
    "ticker": "Thunderbird: New Email - You have a new message from Alice",
    "isClearable": true,
    "time": "1704067200000",
    "silent": "false",
    "urgency": 1,
    "category": "email",
    "richBody": "<b>From:</b> Alice &lt;alice@example.com&gt;",
    "imageData": "iVBORw0KGgoAAAANSUhEUgAAAAUA...",
    "appIcon": "iVBORw0KGgoAAAANSUhEUgAAAAUA...",
    "links": [
      {
        "url": "https://example.com/email/123",
        "title": "View in Browser",
        "start": 0,
        "length": 15
      }
    ],
    "actions": ["Reply", "Mark as Read"],
    "actionButtons": [
      { "id": "reply", "label": "Reply" },
      { "id": "mark_read", "label": "Mark as Read" }
    ]
  }
}
```

#### All Notification Body Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `id` | `string` | **Yes** | - | Unique notification identifier |
| `appName` | `string` | **Yes** | - | Source application name |
| `title` | `string` | **Yes** | - | Notification title/summary |
| `text` | `string` | **Yes** | - | Notification body (plain text) |
| `isClearable` | `bool` | **Yes** | - | User can dismiss notification |
| `ticker` | `string` | No | - | Combined "appName: title - text" |
| `time` | `string` | No | - | Unix timestamp milliseconds (as string) |
| `silent` | `string` | No | `"false"` | `"true"` = preexisting, `"false"` = new |
| `onlyOnce` | `bool` | No | `false` | Show only once (no updates) |
| `urgency` | `u8` | No | `1` | 0=low, 1=normal, 2=critical |
| `category` | `string` | No | - | Notification category |
| `richBody` | `string` | No | - | HTML formatted body |
| `imageData` | `string` | No | - | Base64 PNG image |
| `appIcon` | `string` | No | - | Base64 PNG app icon |
| `senderAvatar` | `string` | No | - | Base64 PNG sender avatar |
| `videoThumbnail` | `string` | No | - | Base64 PNG video thumbnail |
| `links` | `array` | No | - | Clickable link objects |
| `actions` | `array` | No | - | Legacy action labels (strings) |
| `actionButtons` | `array` | No | - | Structured action objects |
| `requestReplyId` | `string` | No | - | UUID for inline reply support |
| `payloadHash` | `string` | No | - | MD5 hash of icon payload |
| `isMessagingApp` | `bool` | No | `false` | Is from messaging app |
| `packageName` | `string` | No | - | Android package name |
| `webUrl` | `string` | No | - | Web interface URL |
| `conversationId` | `string` | No | - | Conversation identifier |
| `isGroupChat` | `bool` | No | `false` | Is group conversation |
| `groupName` | `string` | No | - | Group chat name |
| `hasReplyAction` | `bool` | No | `false` | Supports quick reply |

---

### Cancel Notification Packet

**Type:** `cconnect.notification`

```json
{
  "id": 1704067200000,
  "type": "cconnect.notification",
  "body": {
    "id": "notification-id-123",
    "isCancel": true
  }
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | `string` | **Yes** | Notification ID to cancel |
| `isCancel` | `bool` | **Yes** | Must be `true` |

---

### Request Packet

**Type:** `cconnect.notification.request`

#### Request All Notifications

```json
{
  "id": 1704067200000,
  "type": "cconnect.notification.request",
  "body": {
    "request": true
  }
}
```

#### Dismiss Notification

```json
{
  "id": 1704067200000,
  "type": "cconnect.notification.request",
  "body": {
    "cancel": "notification-id-123"
  }
}
```

---

### Action Packet

**Type:** `cconnect.notification.action`

Sent when user taps an action button on the remote device.

```json
{
  "id": 1704067200000,
  "type": "cconnect.notification.action",
  "body": {
    "key": "desktop-Thunderbird-1704067200000",
    "action": "reply"
  }
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `key` | `string` | **Yes** | Notification ID containing the action |
| `action` | `string` | **Yes** | Action ID from `actionButtons[].id` |

---

### Reply Packet

**Type:** `cconnect.notification.reply`

Sent when user types an inline reply.

```json
{
  "id": 1704067200000,
  "type": "cconnect.notification.reply",
  "body": {
    "requestReplyId": "uuid-string",
    "message": "Thanks for the message!"
  }
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `requestReplyId` | `string` | **Yes** | Reply UUID from notification |
| `message` | `string` | **Yes** | User's reply text |

---

## Field Mappings

### COSMIC Connect to FreeDesktop D-Bus

When sending notifications to the desktop, map fields as follows:

| COSMIC Connect Field | FreeDesktop Method Param | Notes |
|---------------------|-------------------------|-------|
| `appName` | `app_name` (string) | Direct mapping |
| - | `replaces_id` (u32) | 0 for new, tracked ID for updates |
| `appIcon` / app_name | `app_icon` (string) | Icon name or decode base64 |
| `title` | `summary` (string) | Direct mapping |
| `text` / `richBody` | `body` (string) | Sanitize HTML if rich |
| `actionButtons` | `actions` (as) | Flatten to [id, label, id, label, ...] |
| `urgency` | `hints["urgency"]` (y) | Map 0/1/2 |
| `category` | `hints["category"]` (s) | Direct mapping |
| `imageData` | `hints["image-data"]` | Convert to (iiibiiay) struct |
| - | `expire_timeout` (i) | -1 for default |

### FreeDesktop D-Bus to COSMIC Connect

When capturing desktop notifications to send to phone:

| FreeDesktop Field | COSMIC Connect Field | Notes |
|------------------|---------------------|-------|
| `app_name` | `appName` | Direct mapping |
| `summary` | `title` | Direct mapping |
| `body` | `text` | Strip HTML, extract links |
| `body` | `richBody` | Sanitize but preserve |
| `body` | `links` | Extract href URLs |
| `actions[id, label, ...]` | `actionButtons` | Pair into objects |
| `hints["urgency"]` | `urgency` | Direct mapping (u8) |
| `hints["category"]` | `category` | Direct mapping |
| `hints["image-data"]` | `imageData` | Convert to base64 PNG |
| `hints["image-path"]` | `imageData` | Load file, encode as PNG |
| `hints["icon_data"]` | `appIcon` | Convert to base64 PNG |
| `hints["desktop-entry"]` | - | Use to look up app icon |

### Urgency Mapping

| Value | FreeDesktop | COSMIC Connect | Android |
|-------|-------------|----------------|---------|
| 0 | Low | Low | IMPORTANCE_LOW |
| 1 | Normal | Normal (default) | IMPORTANCE_DEFAULT |
| 2 | Critical | Critical | IMPORTANCE_HIGH |

### Category Mapping

Common notification categories (freedesktop.org spec):

| Category | Description |
|----------|-------------|
| `device` | Device status (battery, network) |
| `device.added` | New device connected |
| `device.error` | Device error |
| `device.removed` | Device disconnected |
| `email` | Email notification |
| `email.arrived` | New email received |
| `email.bounced` | Email bounced |
| `im` | Instant message |
| `im.error` | IM error |
| `im.received` | New IM received |
| `network` | Network status |
| `network.connected` | Network connected |
| `network.disconnected` | Network disconnected |
| `network.error` | Network error |
| `presence` | Presence change |
| `presence.offline` | Contact went offline |
| `presence.online` | Contact came online |
| `transfer` | File transfer |
| `transfer.complete` | Transfer completed |
| `transfer.error` | Transfer error |

---

## Rich Content

### HTML Body (`richBody`)

#### Allowed Tags

| Tag | Purpose | Attributes |
|-----|---------|------------|
| `<b>` | Bold text | None |
| `<i>` | Italic text | None |
| `<u>` | Underlined text | None |
| `<a>` | Hyperlink | `href` only |
| `<br>` | Line break | None |
| `<p>` | Paragraph | None |

#### Stripped Tags (Dangerous)

- `<script>`, `<style>`, `<iframe>`, `<object>`, `<embed>`
- `<img>`, `<video>`, `<audio>` (use `imageData`/`videoThumbnail` instead)
- `<link>`, `<meta>`, `<html>`, `<head>`, `<body>`

#### Example

```json
{
  "richBody": "<b>Important:</b> Check <a href=\"https://example.com\">this link</a>"
}
```

---

### Image Data

#### Format

All image fields use **Base64-encoded PNG** format:

```json
{
  "imageData": "iVBORw0KGgoAAAANSUhEUgAAAAUA..."
}
```

#### Image Fields

| Field | Purpose | Max Size |
|-------|---------|----------|
| `imageData` | Notification content image | 256x256 px |
| `appIcon` | Application icon | 128x128 px |
| `senderAvatar` | Sender profile picture | 128x128 px |
| `videoThumbnail` | Video preview thumbnail | 256x256 px |

#### FreeDesktop Image-Data Structure

When receiving `image-data` hint from D-Bus:

```
(width: i32, height: i32, rowstride: i32, has_alpha: bool,
 bits_per_sample: i32, channels: i32, data: Vec<u8>)
```

Convert to PNG for transmission:
1. Parse the struct
2. Handle rowstride padding
3. Convert RGBA/RGB to image
4. Resize if > MAX_DIMENSION (256px)
5. Encode as PNG
6. Base64 encode

---

### Links

#### Link Object Structure

```json
{
  "url": "https://example.com/article",
  "title": "Read More",
  "start": 10,
  "length": 9
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `url` | `string` | **Yes** | URL to open (http/https/mailto only) |
| `title` | `string` | No | Display title for link |
| `start` | `usize` | **Yes** | Character offset in text |
| `length` | `usize` | **Yes** | Length of linked text |

#### Extracting Links

Links can be extracted from:

1. **HTML href attributes** in `richBody`:
   ```html
   <a href="https://example.com">Link Text</a>
   ```

2. **Plain text URLs** in `text`:
   ```
   Check out https://example.com for more info
   ```

3. **Entity-encoded HTML** (Chrome sends this):
   ```
   &lt;a href=&quot;https://example.com&quot;&gt;Link&lt;/a&gt;
   ```

---

### Action Buttons

#### Legacy Format (`actions`)

Simple string array of labels:

```json
{
  "actions": ["Reply", "Mark as Read", "Delete"]
}
```

#### Structured Format (`actionButtons`)

Array of objects with ID and label:

```json
{
  "actionButtons": [
    { "id": "reply", "label": "Reply" },
    { "id": "mark_read", "label": "Mark as Read" },
    { "id": "delete", "label": "Delete" }
  ]
}
```

**Best Practice:** Include both formats for backward compatibility:

```json
{
  "actions": ["Reply", "Mark as Read"],
  "actionButtons": [
    { "id": "reply", "label": "Reply" },
    { "id": "mark_read", "label": "Mark as Read" }
  ]
}
```

---

## Security

### HTML Sanitization

**CRITICAL:** All HTML content MUST be sanitized before display.

#### Implementation (ammonia crate)

```rust
use ammonia::Builder;
use std::collections::HashSet;

pub fn sanitize_html(html: &str) -> String {
    let mut allowed_tags = HashSet::new();
    allowed_tags.insert("b");
    allowed_tags.insert("i");
    allowed_tags.insert("u");
    allowed_tags.insert("a");
    allowed_tags.insert("br");
    allowed_tags.insert("p");

    let mut allowed_attrs = HashSet::new();
    allowed_attrs.insert("href");

    let mut url_schemes = HashSet::new();
    url_schemes.insert("http");
    url_schemes.insert("https");
    url_schemes.insert("mailto");

    Builder::default()
        .tags(allowed_tags)
        .link_rel(Some("noopener noreferrer"))
        .url_schemes(url_schemes)
        .generic_attributes(HashSet::new())
        .tag_attributes(std::iter::once(("a", allowed_attrs)).collect())
        .clean(html)
        .to_string()
}
```

#### Threats Mitigated

| Threat | Mitigation |
|--------|------------|
| XSS via `<script>` | Tag stripped |
| Event handlers (onclick, onerror) | Attributes stripped |
| Content injection (`<iframe>`, `<embed>`) | Tags stripped |
| Style attacks (`<style>`) | Tag stripped |
| Dangerous URLs (javascript:, data:) | Scheme blocked |
| Link hijacking | `rel="noopener noreferrer"` added |

---

### URL Validation

**CRITICAL:** Only allow safe URL schemes.

#### Safe Schemes

```rust
pub fn is_safe_url(url: &str) -> bool {
    let url_lower = url.to_lowercase();
    url_lower.starts_with("https://") ||
    url_lower.starts_with("http://") ||
    url_lower.starts_with("mailto:")
}
```

#### Blocked Schemes

| Scheme | Reason |
|--------|--------|
| `javascript:` | Code execution |
| `data:` | Content injection |
| `vbscript:` | Code execution |
| `file:` | Local file access |
| `ftp:` | Unencrypted protocol |

---

### Entity Decoding

Some applications (notably Chrome) send HTML with entity-encoded tags.

**Example:** `&lt;a href=&quot;https://example.com&quot;&gt;Link&lt;/a&gt;`

**Process:**
1. Decode entities first
2. Then sanitize/extract

```rust
fn decode_entities(text: &str) -> String {
    text
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#58;", ":")  // Colon (decimal)
        .replace("&#x3A;", ":") // Colon (hex)
        .replace("&amp;", "&")  // Must be last
}
```

---

### Image Security

| Concern | Mitigation |
|---------|------------|
| Large images consuming memory | Resize to max 256x256 |
| Malformed image data | Validate dimensions/channels |
| Base64 size bloat (~33%) | Accept tradeoff for simplicity |
| Untrusted image sources | Images are display-only, no execution |

---

## Implementation Checklist

### Component: Notification Plugin (Protocol Layer)

**File:** `cosmic-ext-connect-protocol/src/plugins/notification.rs`

- [ ] Parse all notification body fields
- [ ] Serialize notification to JSON correctly
- [ ] Handle both `cconnect.` and `kdeconnect.` prefixes
- [ ] Implement `create_notification_packet()`
- [ ] Implement `create_cancel_packet()`
- [ ] Implement `create_action_invocation_packet()`
- [ ] Implement `create_dismiss_packet()`
- [ ] Implement `create_request_packet()`
- [ ] Track active notifications by ID
- [ ] Handle cancel/dismiss messages
- [ ] Base64 decode image fields
- [ ] Truncate body if > 2000 chars

### Component: Notification Listener (DBus Capture)

**File:** `cosmic-ext-connect-daemon/src/notification_listener.rs`

- [ ] Connect to session DBus
- [ ] Create MatchRule for `org.freedesktop.Notifications.Notify`
- [ ] Parse all 8 method parameters
- [ ] Extract hints (urgency, category, image-data, etc.)
- [ ] Parse actions into (id, label) pairs
- [ ] Handle image-data struct (7 fields)
- [ ] Handle image-path (load and encode)
- [ ] Apply exclusion filters (CConnect, cosmic-notifications)
- [ ] Truncate body if configured
- [ ] Filter transient/low-urgency if configured

### Component: Image Processing

**File:** `cosmic-ext-connect-daemon/src/notification_image.rs`

- [ ] Convert RGBA image-data to PNG
- [ ] Convert RGB image-data to RGBA then PNG
- [ ] Handle rowstride padding
- [ ] Load image from file path
- [ ] Resize images > MAX_DIMENSION (256px)
- [ ] Maintain aspect ratio when resizing
- [ ] Base64 encode output

### Component: Desktop Notification Display

**File:** `cosmic-ext-connect-daemon/src/cosmic_notifications.rs` (or integration)

- [ ] Receive notification from remote device
- [ ] Sanitize richBody HTML
- [ ] Extract links from body
- [ ] Create D-Bus notification with:
  - [ ] app_name
  - [ ] replaces_id (track for updates)
  - [ ] app_icon (from appIcon or lookup)
  - [ ] summary (from title)
  - [ ] body (sanitized richBody or text)
  - [ ] actions (flatten actionButtons)
  - [ ] hints (urgency, category, image-data)
- [ ] Handle action callbacks
- [ ] Open links in default browser
- [ ] Track notification metadata for callbacks
- [ ] Handle dismissal sync

### Component: Notification Daemon (COSMIC Notifications NG)

**File:** `cosmic-notifications-ng/cosmic-notifications-util/src/sanitizer.rs`

- [ ] Sanitize HTML with ammonia
- [ ] Preserve allowed tags only (b, i, u, a, br, p)
- [ ] Allow only href attribute on a tags
- [ ] Allow only safe URL schemes (http, https, mailto)
- [ ] Add `rel="noopener noreferrer"` to links
- [ ] Strip HTML for plain text fallback
- [ ] Extract href URLs from anchor tags
- [ ] Decode HTML entities before processing

**File:** `cosmic-notifications-ng/cosmic-notifications-util/src/link_detector.rs`

- [ ] Detect URLs in plain text
- [ ] Detect email addresses
- [ ] Create NotificationLink objects
- [ ] Validate URL safety before opening

### Component: Android Side

- [ ] Capture Android notifications via NotificationListenerService
- [ ] Extract all fields (title, text, icon, actions, etc.)
- [ ] Encode images as base64 PNG
- [ ] Generate unique notification IDs
- [ ] Send notification packets
- [ ] Handle incoming notification packets
- [ ] Display remote notifications with proper channel
- [ ] Handle action invocation
- [ ] Handle inline reply
- [ ] Sync dismissal back to desktop

---

## Reference Files

### COSMIC Connect Desktop App

| File | Purpose |
|------|---------|
| `cosmic-ext-connect-protocol/src/plugins/notification.rs` | Protocol layer notification handling |
| `cosmic-ext-connect-daemon/src/notification_listener.rs` | DBus notification capture |
| `cosmic-ext-connect-daemon/src/notification_image.rs` | Image processing for notifications |
| `docs/RICH_NOTIFICATIONS.md` | Rich notification implementation details |

### COSMIC Notifications NG

| File | Purpose |
|------|---------|
| `cosmic-notifications-util/src/sanitizer.rs` | HTML sanitization (ammonia) |
| `cosmic-notifications-util/src/link_detector.rs` | URL detection in text |
| `cosmic-notifications-util/src/urgency.rs` | Urgency level enum |
| `cosmic-notifications-util/src/notification_image.rs` | Image handling |
| `src/subscriptions/notifications.rs` | D-Bus notification subscription |

### External References

| Resource | URL |
|----------|-----|
| FreeDesktop Notification Spec | https://specifications.freedesktop.org/notification-spec/latest/ |
| KDE Connect Protocol | https://valent.andyholmes.ca/documentation/protocol.html |
| Valent (GNOME reference) | https://github.com/andyholmes/valent |
| GSConnect (reference impl) | https://github.com/GSConnect/gnome-shell-extension-gsconnect |
| ammonia (HTML sanitizer) | https://docs.rs/ammonia/latest/ammonia/ |
| linkify (URL detection) | https://docs.rs/linkify/latest/linkify/ |

---

## Appendix A: Complete Notification Example

### Rich Desktop Notification Sent to Phone

```json
{
  "id": 1704067200000,
  "type": "cconnect.notification",
  "body": {
    "id": "desktop-Signal-1704067200000",
    "appName": "Signal",
    "title": "Alice",
    "text": "Check out this article: https://example.com/article",
    "ticker": "Signal: Alice - Check out this article: https://example.com/article",
    "isClearable": true,
    "time": "1704067200000",
    "silent": "false",
    "urgency": 1,
    "category": "im.received",
    "isMessagingApp": true,
    "packageName": "org.thoughtcrime.securesms",
    "conversationId": "alice-123",
    "hasReplyAction": true,
    "requestReplyId": "550e8400-e29b-41d4-a716-446655440000",
    "richBody": "Check out this article: <a href=\"https://example.com/article\">example.com/article</a>",
    "links": [
      {
        "url": "https://example.com/article",
        "title": "example.com/article",
        "start": 24,
        "length": 18
      }
    ],
    "senderAvatar": "iVBORw0KGgoAAAANSUhEUgAAAAgAAAAICAIAAABLbSncAAAADklEQVQI12P4////GQYCABqfAf8vu5hCAAAAAElFTkSuQmCC",
    "actions": ["Reply", "Mark as Read"],
    "actionButtons": [
      { "id": "reply", "label": "Reply" },
      { "id": "mark_read", "label": "Mark as Read" }
    ]
  }
}
```

---

## Appendix B: Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-02-02 | Initial comprehensive specification |

---

*This document is maintained as part of the COSMIC Connect ecosystem. For questions or updates, see the [Contributing Guide](./CONTRIBUTING.md).*
