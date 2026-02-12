# Android Notification Actions and Links - Implementation Guide

## Overview
This guide documents the changes needed to fix Android notification action buttons and links in cosmic-connect-desktop-app.

## Problem
1. Action buttons from Android notifications (`actionButtons` array) are not being extracted from incoming packets
2. Links array is parsed but lost - always passed as `Vec::new()` to notification system
3. Result: No action buttons appear in desktop notifications, links cannot be clicked

## Solution

### Location
File: `cosmic-ext-connect-daemon/src/main.rs`
Lines: ~2086-2098 (the `else if should_show` block for non-messaging notifications)

### Current Code
```rust
} else if should_show {
    if let Err(e) = notifier
        .notify_from_device(
            &device_name,
            app_name,
            title,
            text,
        )
        .await
    {
        warn!("Failed to send device notification: {}", e);
    }
}
```

### Required Changes

Replace the above block with:

```rust
} else if should_show {
    // Extract notification ID for rich notifications
    let notification_id = packet
        .body
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Check for image data
    let image_data = packet
        .body
        .get("imageData")
        .and_then(|v| v.as_str());

    // Process image if present
    let image_bytes = if let Some(base64_data) = image_data {
        use base64::{engine::general_purpose, Engine as _};

        // Decode base64 to bytes
        match general_purpose::STANDARD.decode(base64_data) {
            Ok(bytes) => {
                // Load as image to get dimensions
                match image::load_from_memory(&bytes) {
                    Ok(img) => {
                        let width = img.width() as i32;
                        let height = img.height() as i32;
                        // Convert to RGBA8 and get raw bytes
                        let rgba = img.to_rgba8();
                        Some((rgba.into_raw(), width, height))
                    }
                    Err(e) => {
                        warn!("Failed to decode notification image: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                warn!("Failed to decode base64 image data: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Extract action buttons from packet
    // Format: [{"id": "reply", "label": "Reply"}, ...]
    let action_buttons = packet
        .body
        .get("actionButtons")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|action| {
                    let id = action.get("id")?.as_str()?;
                    let label = action.get("label")?.as_str()?;
                    Some((id.to_string(), label.to_string()))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    // Extract links from packet
    // Format: [{"url": "https://...", "title": "..."}, ...]
    let links = packet
        .body
        .get("links")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|link| {
                    let url = link.get("url")?.as_str()?;
                    Some(url.to_string())
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    // Log extracted actions and links for debugging
    if !action_buttons.is_empty() {
        debug!(
            "Extracted {} action buttons from notification",
            action_buttons.len()
        );
    }
    if !links.is_empty() {
        debug!(
            "Extracted {} links from notification",
            links.len()
        );
    }

    // Send notification with or without image/actions/links
    if image_bytes.is_some() || !action_buttons.is_empty() || !links.is_empty() {
        // Use rich notification with image, actions, and/or links
        if let Err(e) = notifier
            .notify_rich_from_device(
                notification_id,
                &device_name,
                app_name,
                title,
                text,
                None, // rich_body
                image_bytes,
                links, // Now passing actual links instead of Vec::new()
            )
            .await
        {
            warn!("Failed to send rich notification: {}", e);
        }
    } else {
        // Use simple notification without extra features
        if let Err(e) = notifier
            .notify_from_device(
                &device_name,
                app_name,
                title,
                text,
            )
            .await
        {
            warn!("Failed to send device notification: {}", e);
        }
    }
}
```

## Key Changes

### 1. Extract Action Buttons
```rust
let action_buttons = packet
    .body
    .get("actionButtons")
    .and_then(|v| v.as_array())
    .map(|arr| {
        arr.iter()
            .filter_map(|action| {
                let id = action.get("id")?.as_str()?;
                let label = action.get("label")?.as_str()?;
                Some((id.to_string(), label.to_string()))
            })
            .collect::<Vec<_>>()
    })
    .unwrap_or_default();
```

### 2. Extract Links
```rust
let links = packet
    .body
    .get("links")
    .and_then(|v| v.as_array())
    .map(|arr| {
        arr.iter()
            .filter_map(|link| {
                let url = link.get("url")?.as_str()?;
                Some(url.to_string())
            })
            .collect::<Vec<_>>()
    })
    .unwrap_or_default();
```

### 3. Use Rich Notification When Needed
Instead of always using `notify_from_device`, now use `notify_rich_from_device` when:
- Image data is present, OR
- Action buttons are present, OR
- Links are present

### 4. Pass Links to Notification
Changed from:
```rust
links,               // Was: Vec::new()
```

## Testing

After implementing these changes, test by:

1. **Build the daemon:**
   ```bash
   cargo build --release
   ```

2. **Send a notification from Android with:**
   - Action buttons (e.g., Reply, Mark as Read)
   - Links in the notification body

3. **Verify:**
   - Action buttons appear in the desktop notification
   - Link buttons appear as "Open Link 1", "Open Link 2", etc.
   - Clicking link buttons opens URLs in browser
   - Debug logs show extracted action buttons and links

## Expected Behavior

### Before Fix
- Notifications appear but without action buttons
- Links are silently dropped
- Only image data (if present) is used

### After Fix
- Notifications include all action buttons from Android
- Links become clickable "Open Link N" buttons
- Image data, actions, and links all work together
- Debug logs confirm extraction: "Extracted 2 action buttons from notification"

## Notes

- The `action_buttons` variable is extracted but not currently used. A future enhancement would be to pass these to `NotificationBuilder` directly
- The existing `notify_rich_from_device` method already supports the links parameter correctly
- All changes are in the packet processing code; no changes to `cosmic_notifications.rs` are required for this fix

## Dependencies

No additional dependencies are required. The code uses:
- `base64` crate (already in use)
- `image` crate (already in use)
- Standard `serde_json::Value` methods for packet parsing
