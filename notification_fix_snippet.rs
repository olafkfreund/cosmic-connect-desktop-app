// This is the complete replacement for the notification handling block
// in cosmic-ext-connect-daemon/src/main.rs around line 2086
//
// Replace the section from:
//   } else if should_show {
// to the matching closing brace
//
// This code:
// 1. Extracts notification ID for rich notifications
// 2. Processes image data (already present in partial implementation)
// 3. Extracts actionButtons array from packet
// 4. Extracts links array from packet
// 5. Logs debug info when actions/links are found
// 6. Sends rich notification when image/actions/links are present
// 7. Falls back to simple notification otherwise

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
    // Incoming format: [{"id": "reply", "label": "Reply"}, {"id": "mark_read", "label": "Mark as Read"}]
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
    // Incoming format: [{"url": "https://example.com", "title": "Example", "start": 0, "length": 7}]
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
            "Extracted {} action buttons from notification: {:?}",
            action_buttons.len(),
            action_buttons
                .iter()
                .map(|(id, label)| format!("{}:{}", id, label))
                .collect::<Vec<_>>()
        );
    }
    if !links.is_empty() {
        debug!(
            "Extracted {} links from notification: {:?}",
            links.len(),
            links
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
                None, // rich_body - could be enhanced to use packet.body.get("richBody")
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
                None, // rich_body
            )
            .await
        {
            warn!("Failed to send device notification: {}", e);
        }
    }
}
