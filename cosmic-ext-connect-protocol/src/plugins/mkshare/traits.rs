//! Platform abstraction traits for input capture and injection
//!
//! These traits define the interface for platform-specific input handling,
//! allowing the MouseKeyboardShare plugin to work across different
//! display servers (currently Wayland/COSMIC only).

use super::types::{InputEvent, Modifiers, MouseButton, ScreenGeometry};
use crate::Result;
use async_trait::async_trait;
use tokio::sync::broadcast;

/// Trait for capturing global input events
///
/// Implementations provide platform-specific input capture functionality.
/// On Wayland, this is limited due to security model - we can only reliably
/// get cursor position via compositor-specific protocols.
#[async_trait]
pub trait InputCapture: Send + Sync {
    /// Start capturing global input events
    ///
    /// This begins monitoring input and emitting events through the subscription channel.
    /// On Wayland, this may have limited functionality compared to X11.
    async fn start_capture(&mut self) -> Result<()>;

    /// Stop capturing input events
    async fn stop_capture(&mut self) -> Result<()>;

    /// Check if capture is currently active
    fn is_capturing(&self) -> bool;

    /// Get current cursor position
    ///
    /// Returns `None` if position cannot be determined (common on Wayland
    /// without compositor-specific protocols).
    fn cursor_position(&self) -> Option<(i32, i32)>;

    /// Get geometry of all connected screens/monitors
    fn screen_geometry(&self) -> Vec<ScreenGeometry>;

    /// Subscribe to captured input events
    ///
    /// Returns a broadcast receiver that will receive all captured input events.
    fn subscribe(&self) -> broadcast::Receiver<InputEvent>;
}

/// Trait for injecting input events into the system
///
/// Implementations use uinput (Linux) to create virtual input devices
/// and inject mouse/keyboard events.
#[async_trait]
pub trait InputInjection: Send + Sync {
    /// Initialize the virtual input device
    ///
    /// Creates uinput virtual devices for mouse and keyboard.
    /// Requires appropriate permissions (user in `input` group or udev rules).
    async fn initialize(&mut self) -> Result<()>;

    /// Check if the virtual device is initialized and ready
    fn is_initialized(&self) -> bool;

    /// Inject relative mouse movement
    ///
    /// Moves the cursor by the specified delta.
    async fn inject_mouse_move(&self, dx: i32, dy: i32) -> Result<()>;

    /// Inject absolute mouse position
    ///
    /// Moves the cursor to the specified absolute position.
    /// Note: Absolute positioning may not work on all compositors.
    async fn inject_mouse_position(&self, x: i32, y: i32) -> Result<()>;

    /// Inject mouse button event
    async fn inject_mouse_button(&self, button: MouseButton, pressed: bool) -> Result<()>;

    /// Inject mouse click (press and release)
    async fn inject_mouse_click(&self, button: MouseButton) -> Result<()>;

    /// Inject keyboard key event
    async fn inject_key(&self, keycode: u16, pressed: bool, modifiers: Modifiers) -> Result<()>;

    /// Inject key click (press and release)
    async fn inject_key_click(&self, keycode: u16, modifiers: Modifiers) -> Result<()>;

    /// Inject scroll event
    async fn inject_scroll(&self, dx: f64, dy: f64) -> Result<()>;

    /// Cleanup and release virtual devices
    async fn cleanup(&mut self) -> Result<()>;
}

/// Combined trait for backends that support both capture and injection
///
/// Most platform backends will implement both traits together.
pub trait InputBackend: InputCapture + InputInjection {}

// Auto-implement InputBackend for any type that implements both traits
impl<T: InputCapture + InputInjection> InputBackend for T {}

/// Factory for creating platform-appropriate input backends
pub struct InputBackendFactory;

impl InputBackendFactory {
    /// Detect the current display server and create appropriate backend
    pub async fn create() -> Result<Box<dyn InputBackend>> {
        // COSMIC Desktop uses Wayland exclusively
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            Ok(Box::new(super::wayland::WaylandInputBackend::new().await?))
        } else {
            Err(crate::ProtocolError::Plugin(
                "No supported display server detected. COSMIC Connect requires Wayland.".into(),
            ))
        }
    }

    /// Check if a compatible display server is available
    pub fn is_supported() -> bool {
        std::env::var("WAYLAND_DISPLAY").is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_detects_wayland() {
        // This test will pass or fail based on actual environment
        let supported = InputBackendFactory::is_supported();
        // Just ensure it doesn't panic
        let _ = supported;
    }
}
