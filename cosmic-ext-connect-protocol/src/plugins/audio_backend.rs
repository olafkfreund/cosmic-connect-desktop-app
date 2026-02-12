//! Audio Backend for System Volume Control
//!
//! Provides PipeWire/WirePlumber integration for volume control using wpctl CLI.
//! Falls back gracefully if wpctl is not available.

use std::process::Command;
use tracing::{debug, warn};

/// Represents an audio sink (output device)
#[derive(Debug, Clone)]
pub struct AudioSink {
    /// Node ID used by PipeWire/WirePlumber
    pub id: u32,
    /// Human-readable name/description
    pub name: String,
    /// Current volume (0-100)
    pub volume: i32,
    /// Whether the sink is muted
    pub muted: bool,
    /// Whether this is the default sink
    pub is_default: bool,
    /// Maximum volume (typically 100, but can be higher for boost)
    pub max_volume: i32,
}

/// Audio backend using wpctl (WirePlumber CLI)
pub struct AudioBackend;

impl AudioBackend {
    /// Check if wpctl is available
    pub fn is_available() -> bool {
        Command::new("wpctl")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// List all audio sinks
    pub fn list_sinks() -> Vec<AudioSink> {
        let mut sinks = Vec::new();

        // Get wpctl status output
        let output = match Command::new("wpctl").arg("status").output() {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
            _ => {
                warn!("Failed to run wpctl status");
                return sinks;
            }
        };

        // Parse sinks section
        let mut in_audio_sinks = false;
        for line in output.lines() {
            // Detect start of Audio Sinks section
            if line.contains("├─ Sinks:") || line.contains("└─ Sinks:") {
                in_audio_sinks = true;
                continue;
            }

            // Detect end of Sinks section (next section starts)
            if in_audio_sinks
                && (line.contains("├─ Sources:")
                    || line.contains("├─ Filters:")
                    || line.contains("└─ Streams:"))
            {
                break;
            }

            // Parse sink line
            if in_audio_sinks && line.contains(".") {
                if let Some(sink) = Self::parse_sink_line(line) {
                    sinks.push(sink);
                }
            }
        }

        // Get volume/mute status for each sink
        for sink in &mut sinks {
            if let Some((vol, muted)) = Self::get_sink_volume(sink.id) {
                sink.volume = vol;
                sink.muted = muted;
            }
        }

        debug!("Found {} audio sinks", sinks.len());
        sinks
    }

    /// Parse a sink line from wpctl status output
    /// Example: " │  *   50. Realtek USB Audio Front Speaker     [vol: 1.00]"
    fn parse_sink_line(line: &str) -> Option<AudioSink> {
        // Check if this is the default sink
        let is_default = line.contains("*");

        // Find the node ID (number before the dot)
        let trimmed = line.trim_start_matches(|c: char| !c.is_ascii_digit());
        let dot_pos = trimmed.find('.')?;
        let id: u32 = trimmed[..dot_pos].trim().parse().ok()?;

        // Find the name (between dot and [vol:)
        let after_dot = &trimmed[dot_pos + 1..];
        let name_end = after_dot.find("[vol:").unwrap_or(after_dot.len());
        let name = after_dot[..name_end].trim().to_string();

        if name.is_empty() {
            return None;
        }

        // Parse volume from [vol: X.XX]
        let volume = if let Some(vol_start) = after_dot.find("[vol:") {
            let vol_str = &after_dot[vol_start + 5..];
            if let Some(vol_end) = vol_str.find(']') {
                vol_str[..vol_end]
                    .trim()
                    .parse::<f32>()
                    .map(|v| (v * 100.0) as i32)
                    .unwrap_or(100)
            } else {
                100
            }
        } else {
            100
        };

        Some(AudioSink {
            id,
            name,
            volume,
            muted: false, // Will be updated by get_sink_volume
            is_default,
            max_volume: 150, // Allow volume boost
        })
    }

    /// Get volume and mute status for a specific sink
    fn get_sink_volume(id: u32) -> Option<(i32, bool)> {
        let output = Command::new("wpctl")
            .args(["get-volume", &id.to_string()])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Output format: "Volume: 1.00" or "Volume: 0.50 [MUTED]"
        let muted = stdout.contains("[MUTED]");

        let volume = stdout
            .strip_prefix("Volume:")
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.parse::<f32>().ok())
            .map(|v| (v * 100.0) as i32)
            .unwrap_or(100);

        Some((volume, muted))
    }

    /// Set volume for a sink (0-150, allows boost)
    pub fn set_volume(id: u32, volume: i32) -> bool {
        let volume = volume.clamp(0, 150);
        let vol_str = format!("{}%", volume);

        debug!("Setting volume for sink {} to {}", id, vol_str);

        Command::new("wpctl")
            .args(["set-volume", &id.to_string(), &vol_str])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Set mute status for a sink
    pub fn set_mute(id: u32, muted: bool) -> bool {
        let mute_arg = if muted { "1" } else { "0" };

        debug!("Setting mute for sink {} to {}", id, muted);

        Command::new("wpctl")
            .args(["set-mute", &id.to_string(), mute_arg])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Get the default sink ID
    pub fn get_default_sink_id() -> Option<u32> {
        Self::list_sinks()
            .into_iter()
            .find(|s| s.is_default)
            .map(|s| s.id)
    }

    /// Find sink by name (partial match)
    pub fn find_sink_by_name(name: &str) -> Option<AudioSink> {
        Self::list_sinks()
            .into_iter()
            .find(|s| s.name.to_lowercase().contains(&name.to_lowercase()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sink_line() {
        let line = " │  *   50. Realtek USB Audio Front Speaker     [vol: 1.00]";
        let sink = AudioBackend::parse_sink_line(line).unwrap();
        assert_eq!(sink.id, 50);
        assert_eq!(sink.name, "Realtek USB Audio Front Speaker");
        assert_eq!(sink.volume, 100);
        assert!(sink.is_default);
    }

    #[test]
    fn test_parse_sink_line_non_default() {
        let line = " │      71. Realtek USB Audio Front Headphones  [vol: 0.75]";
        let sink = AudioBackend::parse_sink_line(line).unwrap();
        assert_eq!(sink.id, 71);
        assert_eq!(sink.name, "Realtek USB Audio Front Headphones");
        assert_eq!(sink.volume, 75);
        assert!(!sink.is_default);
    }
}
