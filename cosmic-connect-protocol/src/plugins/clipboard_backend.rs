//! Clipboard Backend
//!
//! Provides system clipboard access via Wayland (wl-copy/wl-paste) or X11 (xclip).
//! Automatically detects the session type and uses the appropriate backend.
//!
//! ## Session Detection
//!
//! The backend checks the `XDG_SESSION_TYPE` environment variable:
//! - `wayland` → Use wl-copy/wl-paste
//! - `x11` → Use xclip
//! - Other/missing → Try Wayland first, fall back to X11
//!
//! ## Command Requirements
//!
//! - Wayland: `wl-copy`, `wl-paste` (from wl-clipboard package)
//! - X11: `xclip` (from xclip package)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use cosmic_connect_core::plugins::clipboard_backend::ClipboardBackend;
//!
//! let mut backend = ClipboardBackend::new();
//!
//! // Read clipboard
//! if let Some(content) = backend.read().await {
//!     println!("Clipboard: {}", content);
//! }
//!
//! // Write clipboard
//! backend.write("Hello, World!").await;
//! ```

use std::env;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::{debug, warn};

/// Session type for clipboard operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Wayland,
    X11,
    Unknown,
}

impl SessionType {
    /// Detect session type from environment
    pub fn detect() -> Self {
        match env::var("XDG_SESSION_TYPE").as_deref() {
            Ok("wayland") => Self::Wayland,
            Ok("x11") => Self::X11,
            _ => {
                // Check for Wayland display
                if env::var("WAYLAND_DISPLAY").is_ok() {
                    Self::Wayland
                } else if env::var("DISPLAY").is_ok() {
                    Self::X11
                } else {
                    Self::Unknown
                }
            }
        }
    }
}

/// System clipboard backend
///
/// Provides read/write access to the system clipboard using
/// session-appropriate commands.
pub struct ClipboardBackend {
    session_type: SessionType,
}

impl ClipboardBackend {
    /// Create a new clipboard backend
    ///
    /// Automatically detects session type from environment.
    pub fn new() -> Self {
        let session_type = SessionType::detect();
        debug!("Clipboard backend using session type: {:?}", session_type);
        Self { session_type }
    }

    /// Read text from system clipboard
    ///
    /// Returns `Some(content)` if clipboard has text content,
    /// `None` if clipboard is empty or an error occurred.
    pub async fn read(&self) -> Option<String> {
        match self.session_type {
            SessionType::Wayland => self.read_wayland().await,
            SessionType::X11 => self.read_x11().await,
            SessionType::Unknown => {
                // Try Wayland first, fall back to X11
                if let Some(content) = self.read_wayland().await {
                    return Some(content);
                }
                self.read_x11().await
            }
        }
    }

    /// Write text to system clipboard
    ///
    /// Returns `true` if successful, `false` otherwise.
    pub async fn write(&self, content: &str) -> bool {
        match self.session_type {
            SessionType::Wayland => self.write_wayland(content).await,
            SessionType::X11 => self.write_x11(content).await,
            SessionType::Unknown => {
                // Try Wayland first, fall back to X11
                if self.write_wayland(content).await {
                    return true;
                }
                self.write_x11(content).await
            }
        }
    }

    /// Check if clipboard commands are available
    pub async fn is_available(&self) -> bool {
        match self.session_type {
            SessionType::Wayland => Self::command_exists("wl-paste").await,
            SessionType::X11 => Self::command_exists("xclip").await,
            SessionType::Unknown => {
                Self::command_exists("wl-paste").await || Self::command_exists("xclip").await
            }
        }
    }

    /// Read clipboard using wl-paste (Wayland)
    async fn read_wayland(&self) -> Option<String> {
        let output = Command::new("wl-paste")
            .arg("--no-newline")
            .arg("--type")
            .arg("text/plain")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await
            .ok()?;

        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout).to_string();
            if !content.is_empty() {
                debug!("Read {} chars from Wayland clipboard", content.len());
                return Some(content);
            }
        }

        None
    }

    /// Read clipboard using xclip (X11)
    async fn read_x11(&self) -> Option<String> {
        let output = Command::new("xclip")
            .arg("-selection")
            .arg("clipboard")
            .arg("-o")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await
            .ok()?;

        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout).to_string();
            if !content.is_empty() {
                debug!("Read {} chars from X11 clipboard", content.len());
                return Some(content);
            }
        }

        None
    }

    /// Write clipboard using wl-copy (Wayland)
    async fn write_wayland(&self, content: &str) -> bool {
        let mut child = match Command::new("wl-copy")
            .arg("--type")
            .arg("text/plain")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                warn!("Failed to spawn wl-copy: {}", e);
                return false;
            }
        };

        if let Some(stdin) = child.stdin.as_mut() {
            if stdin.write_all(content.as_bytes()).await.is_err() {
                warn!("Failed to write to wl-copy stdin");
                return false;
            }
        }

        match child.wait().await {
            Ok(status) if status.success() => {
                debug!("Wrote {} chars to Wayland clipboard", content.len());
                true
            }
            Ok(status) => {
                warn!("wl-copy exited with status: {}", status);
                false
            }
            Err(e) => {
                warn!("Failed to wait for wl-copy: {}", e);
                false
            }
        }
    }

    /// Write clipboard using xclip (X11)
    async fn write_x11(&self, content: &str) -> bool {
        let mut child = match Command::new("xclip")
            .arg("-selection")
            .arg("clipboard")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                warn!("Failed to spawn xclip: {}", e);
                return false;
            }
        };

        if let Some(stdin) = child.stdin.as_mut() {
            if stdin.write_all(content.as_bytes()).await.is_err() {
                warn!("Failed to write to xclip stdin");
                return false;
            }
        }

        match child.wait().await {
            Ok(status) if status.success() => {
                debug!("Wrote {} chars to X11 clipboard", content.len());
                true
            }
            Ok(status) => {
                warn!("xclip exited with status: {}", status);
                false
            }
            Err(e) => {
                warn!("Failed to wait for xclip: {}", e);
                false
            }
        }
    }

    /// Check if a command exists
    async fn command_exists(cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

impl Default for ClipboardBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_type_detection() {
        // Just verify detection doesn't panic
        let session_type = SessionType::detect();
        assert!(matches!(
            session_type,
            SessionType::Wayland | SessionType::X11 | SessionType::Unknown
        ));
    }

    #[test]
    fn test_backend_creation() {
        let backend = ClipboardBackend::new();
        assert!(matches!(
            backend.session_type,
            SessionType::Wayland | SessionType::X11 | SessionType::Unknown
        ));
    }

    #[tokio::test]
    async fn test_is_available() {
        let backend = ClipboardBackend::new();
        // Just verify this doesn't panic
        let _available = backend.is_available().await;
    }

    #[tokio::test]
    #[ignore = "Requires clipboard access"]
    async fn test_read_clipboard() {
        let backend = ClipboardBackend::new();
        // This test is ignored by default as it requires actual clipboard access
        let _content = backend.read().await;
    }

    #[tokio::test]
    #[ignore = "Requires clipboard access"]
    async fn test_write_read_clipboard() {
        let backend = ClipboardBackend::new();
        let test_content = "cosmic-connect-test-content";

        // Write to clipboard
        let written = backend.write(test_content).await;
        if !written {
            // Skip if clipboard not available
            return;
        }

        // Read back
        let content = backend.read().await;
        assert_eq!(content, Some(test_content.to_string()));
    }
}
