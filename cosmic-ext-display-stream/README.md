# cosmic-ext-display-stream

Extended display streaming to Android tablets for COSMIC Desktop.

## Overview

This crate implements screen capture and streaming functionality specifically designed to stream a virtual HDMI display output to Android tablets using the KDE Connect protocol.

## Phase 1: Screen Capture (Current Implementation)

The first phase focuses on capturing video from HDMI dummy plug outputs:

- **xdg-desktop-portal Integration**: Request screen capture permission through the COSMIC portal
- **PipeWire Streaming**: Connect to PipeWire streams for raw video data
- **HDMI Dummy Filtering**: Automatically detect and filter for HDMI dummy displays
- **Frame Reception**: Receive raw video frames from the compositor

### Status

The basic architecture is in place with the following components:

- ✅ Error handling with `thiserror`
- ✅ Output information and HDMI dummy detection
- ✅ Screen capture session management
- ✅ PipeWire stream structure
- ⚠️  Full ashpd portal integration (in progress)
- ⚠️  PipeWire frame processing (in progress)

## Architecture

```
┌─────────────────────┐
│  COSMIC Desktop     │
│  (Wayland)          │
└──────┬──────────────┘
       │
       │ (Screen Capture Request)
       ▼
┌─────────────────────┐
│ xdg-desktop-portal  │
│ (Permission Dialog) │
└──────┬──────────────┘
       │
       │ (Approved)
       ▼
┌─────────────────────┐
│    PipeWire         │
│  (Video Frames)     │
└──────┬──────────────┘
       │
       │ (Raw Frames)
       ▼
┌─────────────────────┐
│ ScreenCapture       │
│ (This Crate)        │
└─────────────────────┘
```

## Future Phases

### Phase 2: Video Encoding
- H.264 encoding with hardware acceleration (VAAPI/NVENC)
- Configurable quality and bitrate settings
- Frame buffering and rate limiting

### Phase 3: Network Streaming
- Stream encoded video over KDE Connect protocol
- Custom packet format for display streaming
- Latency optimization and adaptive bitrate

## Requirements

- COSMIC Desktop (Wayland compositor)
- xdg-desktop-portal-cosmic
- PipeWire runtime
- HDMI dummy plug hardware (or virtual display)

## Usage Example

```rust
use cosmic_display_stream::capture::ScreenCapture;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a screen capture session for HDMI-2 (dummy plug)
    let mut capture = ScreenCapture::new("HDMI-2").await?;

    // Get output information
    if let Some(output) = capture.get_output_info() {
        println!("Capturing: {}", output.description());
    }

    // Start capturing frames
    let mut frame_stream = capture.start_capture().await?;

    // Process frames
    while let Some(frame) = frame_stream.next().await {
        println!("Received frame: {}x{} @ {}",
            frame.width, frame.height, frame.timestamp);
        // TODO: Encode and stream frame
    }

    // Stop capture
    capture.stop_capture().await?;

    Ok(())
}
```

## Development Status

This crate is under active development as part of the COSMIC Connect project. The screen capture functionality is being implemented in phases:

1. **Phase 1 (Current)**: Basic screen capture infrastructure and xdg-desktop-portal integration
2. **Phase 2 (Planned)**: Video encoding with hardware acceleration
3. **Phase 3 (Planned)**: Network streaming over KDE Connect protocol

## Contributing

This crate is part of the COSMIC Connect project. See the main repository for contribution guidelines.

## License

GPL-3.0-or-later
