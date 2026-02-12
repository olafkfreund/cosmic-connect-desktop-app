# Final Changes Needed to Fix Notification Actions and Links

## Current Status
The file `cosmic-ext-connect-daemon/src/main.rs` already has a partial implementation that extracts and processes images from Android notifications. However, **action buttons and links are still missing**.

## Changes Required

### Step 1: Add Code to Extract Actions and Links

Insert the following code **after the image processing block** and **before the "Send notification" comment** (around line 2130):

```rust
// Extract action buttons from packet
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
```

### Step 2: Change the Notification Send Logic

Find this line (around line 2143):
```rust
Vec::new(), // links
```

Replace it with:
```rust
links, // Now passing actual links
```

### Step 3: Update the Condition for Rich Notification

Find this line (around line 2131):
```rust
if image_bytes.is_some() {
```

Replace it with:
```rust
if image_bytes.is_some() || !action_buttons.is_empty() || !links.is_empty() {
```

This ensures rich notifications are used whenever we have images, actions, OR links.

### Step 4: Update the Comment

Change this comment (around line 2130):
```rust
// Send notification with or without image
```

To:
```rust
// Send notification with or without image/actions/links
```

And change this comment:
```rust
// Use rich notification with image
```

To:
```rust
// Use rich notification with image, actions, and/or links
```

And change this comment:
```rust
// Use simple notification without image
```

To:
```rust
// Use simple notification without extra features
```

## Complete "After" Code Block

Here's what the complete block should look like after all changes (around lines 2130-2165):

```rust
                                            };

                                            // Extract action buttons from packet
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
                                                        links, // Now passing actual links
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
                                                        None, // rich_body
                                                    )
                                                    .await
                                                {
                                                    warn!("Failed to send device notification: {}", e);
                                                }
                                            }
```

## Quick Reference: Line Numbers (Approximate)

Based on the current git diff:
- **Line 2129:** End of image processing (the closing `};`)
- **Line 2130:** INSERT action buttons and links extraction code here
- **Line 2131 (after insertion):** Change `if image_bytes.is_some()` condition
- **Line 2143 (after insertion):** Change `Vec::new()` to `links`

## How to Apply These Changes

### Option 1: Manual Editing
1. Open `cosmic-ext-connect-daemon/src/main.rs` in your editor
2. Find line 2129 (search for the last line of image processing)
3. Insert the action buttons and links extraction code
4. Make the three changes described above
5. Save the file

### Option 2: Using the Provided Snippet File
1. Open `notification_fix_snippet.rs` in this directory
2. Copy the complete block starting with `} else if should_show {`
3. Replace the corresponding block in `main.rs` (lines ~2086-2165)

### Option 3: Create a Git Patch
```bash
# After making the changes manually, create a patch
git add cosmic-ext-connect-daemon/src/main.rs
git diff --cached > notification_fix.patch

# To apply on another machine
git apply notification_fix.patch
```

## Testing After Changes

```bash
# Build
cargo build --release

# Check for errors
cargo clippy

# Run tests (if any)
cargo test

# Install and test with Android device
systemctl --user restart cosmic-ext-connect-daemon

# Watch logs
journalctl -u cosmic-ext-connect-daemon -f | grep "Extracted"
```

## Expected Log Output

After the changes, when a notification with actions or links arrives from Android, you should see:

```
DEBUG Extracted 2 action buttons from notification
DEBUG Extracted 1 links from notification
```

## Verification Checklist

- [ ] Code compiles without errors
- [ ] Code passes `cargo clippy` without warnings
- [ ] Notifications from Android appear on desktop
- [ ] Debug logs show extracted actions and links
- [ ] Links create "Open Link N" buttons in notifications
- [ ] Clicking link buttons opens URLs in browser

## Troubleshooting

**Problem:** Code doesn't compile
- **Solution:** Check that the image processing code is complete and properly closed with `};`

**Problem:** No debug logs appear
- **Solution:** Check that Android is sending `actionButtons` and `links` in the packet

**Problem:** Links don't work when clicked
- **Solution:** This is a known limitation - action callbacks need to be implemented separately (see FIX_SUMMARY.md)

**Problem:** File keeps getting modified
- **Solution:** A formatter or LSP might be running. Make all changes at once and save.
