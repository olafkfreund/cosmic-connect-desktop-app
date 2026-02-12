//! Wayland/COSMIC input backend implementation
//!
//! This module implements input injection using Linux uinput via the
//! `mouse-keyboard-input` crate. Input capture on Wayland is limited
//! due to the security model - we rely on compositor protocols where available.

use super::traits::{InputCapture, InputInjection};
use super::types::{InputEvent, Modifiers, MouseButton, ScreenGeometry};
use crate::{ProtocolError, Result};
use async_trait::async_trait;
use mouse_keyboard_input::VirtualDevice;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

/// Wayland input backend using uinput for injection
///
/// On Wayland, we cannot capture global input due to security restrictions.
/// Input injection is done via Linux uinput (virtual input device).
pub struct WaylandInputBackend {
    /// Virtual input device for injection (mouse + keyboard)
    virtual_device: Arc<RwLock<Option<VirtualDevice>>>,
    /// Whether capture is active (limited on Wayland)
    capturing: AtomicBool,
    /// Whether the backend is initialized
    initialized: AtomicBool,
    /// Event sender for captured events
    event_tx: broadcast::Sender<InputEvent>,
    /// Cached screen geometry
    screens: Arc<RwLock<Vec<ScreenGeometry>>>,
    /// Estimated cursor position (updated on our own injections)
    cursor_pos: Arc<RwLock<Option<(i32, i32)>>>,
}

impl WaylandInputBackend {
    /// Create a new Wayland input backend
    pub async fn new() -> Result<Self> {
        let (event_tx, _) = broadcast::channel(1024);

        Ok(Self {
            virtual_device: Arc::new(RwLock::new(None)),
            capturing: AtomicBool::new(false),
            initialized: AtomicBool::new(false),
            event_tx,
            screens: Arc::new(RwLock::new(Vec::new())),
            cursor_pos: Arc::new(RwLock::new(None)),
        })
    }

    /// Initialize the virtual input device
    fn init_virtual_device(&self) -> Result<()> {
        let mut device_guard = self
            .virtual_device
            .write()
            .map_err(|_| ProtocolError::Plugin("Lock poisoned".into()))?;

        if device_guard.is_some() {
            return Ok(());
        }

        match VirtualDevice::default() {
            Ok(dev) => {
                info!("Created virtual input device for Wayland backend");
                *device_guard = Some(dev);
                self.initialized.store(true, Ordering::SeqCst);
                Ok(())
            }
            Err(e) => {
                error!("Failed to create virtual input device: {}", e);
                Err(ProtocolError::Plugin(format!(
                    "Failed to create uinput device: {}. Ensure user is in 'input' group.",
                    e
                )))
            }
        }
    }

    /// Get screen geometry from Wayland compositor
    ///
    /// ## Implementation Notes
    ///
    /// Full screen detection on Wayland requires compositor-specific protocols:
    /// - **wl_output**: Wayland core protocol for screen info (requires Wayland connection)
    /// - **COSMIC DBus API**: COSMIC-specific D-Bus interface for display management
    /// - **wlr-output-management**: wlroots protocol for output configuration
    ///
    /// Current implementation returns placeholder geometry. Screen configuration
    /// should be provided via MkShareConfig instead of runtime detection.
    ///
    /// ## Future Work
    ///
    /// - Integrate with COSMIC compositor D-Bus API when available
    /// - Use wayland-client to enumerate wl_output objects
    /// - Support multi-monitor configurations with proper coordinates
    fn detect_screens(&self) -> Vec<ScreenGeometry> {
        // Placeholder: Return default screen geometry
        // Screen configuration should be provided via MkShareConfig.local_geometry
        vec![ScreenGeometry::new(0, 0, 1920, 1080, "WAYLAND-1")]
    }

    /// Execute an operation with the virtual device
    fn with_device<F, T>(&self, op: F) -> Result<T>
    where
        F: FnOnce(&mut VirtualDevice) -> std::result::Result<T, Box<dyn std::error::Error>>,
    {
        let mut device_guard = self
            .virtual_device
            .write()
            .map_err(|_| ProtocolError::Plugin("Lock poisoned".into()))?;

        let device = device_guard
            .as_mut()
            .ok_or_else(|| ProtocolError::Plugin("Virtual device not initialized".into()))?;

        op(device).map_err(|e| ProtocolError::Plugin(format!("Input injection failed: {}", e)))
    }
}

#[async_trait]
impl InputCapture for WaylandInputBackend {
    async fn start_capture(&mut self) -> Result<()> {
        // On Wayland, we cannot truly capture global input
        // Mark as capturing but functionality is limited
        warn!("Input capture on Wayland is limited. Edge detection relies on cursor position polling.");
        self.capturing.store(true, Ordering::SeqCst);

        // Detect and cache screens
        if let Ok(mut guard) = self.screens.write() {
            *guard = self.detect_screens();
        }

        Ok(())
    }

    async fn stop_capture(&mut self) -> Result<()> {
        self.capturing.store(false, Ordering::SeqCst);
        debug!("Stopped Wayland input capture");
        Ok(())
    }

    fn is_capturing(&self) -> bool {
        self.capturing.load(Ordering::SeqCst)
    }

    fn cursor_position(&self) -> Option<(i32, i32)> {
        // Return our tracked position (updated when we inject events)
        // True cursor position requires compositor-specific protocols
        self.cursor_pos.read().ok().and_then(|guard| *guard)
    }

    fn screen_geometry(&self) -> Vec<ScreenGeometry> {
        self.screens
            .read()
            .ok()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    fn subscribe(&self) -> broadcast::Receiver<InputEvent> {
        self.event_tx.subscribe()
    }
}

#[async_trait]
impl InputInjection for WaylandInputBackend {
    async fn initialize(&mut self) -> Result<()> {
        self.init_virtual_device()
    }

    fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::SeqCst)
    }

    async fn inject_mouse_move(&self, dx: i32, dy: i32) -> Result<()> {
        debug!("Injecting mouse move: dx={}, dy={}", dx, dy);
        self.with_device(|dev| dev.smooth_move_mouse(dx, dy))?;

        // Update our cursor position estimate
        if let Ok(mut guard) = self.cursor_pos.write() {
            if let Some((x, y)) = *guard {
                *guard = Some((x + dx, y + dy));
            }
        }

        Ok(())
    }

    async fn inject_mouse_position(&self, x: i32, y: i32) -> Result<()> {
        debug!("Injecting absolute mouse position: x={}, y={}", x, y);

        // uinput relative devices can't do absolute positioning directly
        // We'd need to calculate delta from current position
        // For now, update our tracked position
        if let Ok(mut guard) = self.cursor_pos.write() {
            *guard = Some((x, y));
        }

        warn!("Absolute mouse positioning not fully implemented on Wayland");
        Ok(())
    }

    async fn inject_mouse_button(&self, button: MouseButton, pressed: bool) -> Result<()> {
        let btn_code = button.to_linux_code();
        debug!(
            "Injecting mouse button: {:?} (code={}) pressed={}",
            button, btn_code, pressed
        );

        self.with_device(|dev| {
            if pressed {
                dev.press(btn_code)
            } else {
                dev.release(btn_code)
            }
        })
    }

    async fn inject_mouse_click(&self, button: MouseButton) -> Result<()> {
        let btn_code = button.to_linux_code();
        debug!("Injecting mouse click: {:?} (code={})", button, btn_code);
        self.with_device(|dev| dev.click(btn_code))
    }

    async fn inject_key(&self, keycode: u16, pressed: bool, modifiers: Modifiers) -> Result<()> {
        debug!(
            "Injecting key: code={} pressed={} modifiers={:?}",
            keycode, pressed, modifiers
        );

        // Handle modifier keys
        if modifiers.any() {
            self.inject_modifiers(&modifiers, pressed)?;
        }

        self.with_device(|dev| {
            if pressed {
                dev.press(keycode)
            } else {
                dev.release(keycode)
            }
        })
    }

    async fn inject_key_click(&self, keycode: u16, modifiers: Modifiers) -> Result<()> {
        debug!(
            "Injecting key click: code={} modifiers={:?}",
            keycode, modifiers
        );

        // Press modifiers
        if modifiers.any() {
            self.inject_modifiers(&modifiers, true)?;
        }

        // Click key
        self.with_device(|dev| dev.click(keycode))?;

        // Release modifiers
        if modifiers.any() {
            self.inject_modifiers(&modifiers, false)?;
        }

        Ok(())
    }

    async fn inject_scroll(&self, dx: f64, dy: f64) -> Result<()> {
        debug!("Injecting scroll: dx={}, dy={}", dx, dy);
        self.with_device(|dev| dev.smooth_scroll(dx as i32, dy as i32))
    }

    async fn cleanup(&mut self) -> Result<()> {
        debug!("Cleaning up Wayland input backend");

        // Drop virtual device
        if let Ok(mut guard) = self.virtual_device.write() {
            *guard = None;
        }

        self.initialized.store(false, Ordering::SeqCst);
        self.capturing.store(false, Ordering::SeqCst);

        info!("Wayland input backend cleaned up");
        Ok(())
    }
}

impl WaylandInputBackend {
    /// Press or release modifier keys
    fn inject_modifiers(&self, modifiers: &Modifiers, pressed: bool) -> Result<()> {
        use mouse_keyboard_input::{KEY_LEFTALT, KEY_LEFTCTRL, KEY_LEFTMETA, KEY_LEFTSHIFT};

        self.with_device(|dev| {
            let modifier_keys = [
                (modifiers.shift, KEY_LEFTSHIFT),
                (modifiers.ctrl, KEY_LEFTCTRL),
                (modifiers.alt, KEY_LEFTALT),
                (modifiers.meta, KEY_LEFTMETA),
            ];

            for (is_active, keycode) in modifier_keys {
                if is_active {
                    if pressed {
                        dev.press(keycode)?;
                    } else {
                        dev.release(keycode)?;
                    }
                }
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wayland_backend_creation() {
        let backend = WaylandInputBackend::new().await;
        assert!(backend.is_ok());

        let backend = backend.unwrap();
        assert!(!backend.is_initialized());
        assert!(!backend.is_capturing());
    }

    #[tokio::test]
    async fn test_screen_geometry_default() {
        let backend = WaylandInputBackend::new().await.unwrap();
        let screens = backend.detect_screens();

        assert!(!screens.is_empty());
        assert_eq!(screens[0].name, "WAYLAND-1");
    }

    #[tokio::test]
    async fn test_cursor_position_tracking() {
        let backend = WaylandInputBackend::new().await.unwrap();

        // Initially no position
        assert!(backend.cursor_position().is_none());

        // Set position manually for testing
        if let Ok(mut guard) = backend.cursor_pos.write() {
            *guard = Some((100, 200));
        }

        assert_eq!(backend.cursor_position(), Some((100, 200)));
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let backend = WaylandInputBackend::new().await.unwrap();
        let mut receiver = backend.subscribe();

        // Send an event
        let _ = backend
            .event_tx
            .send(InputEvent::MouseMove { dx: 10, dy: 20 });

        // Receive it
        let event = receiver.try_recv();
        assert!(event.is_ok());

        if let Ok(InputEvent::MouseMove { dx, dy }) = event {
            assert_eq!(dx, 10);
            assert_eq!(dy, 20);
        } else {
            panic!("Unexpected event type");
        }
    }

    // Note: Tests requiring actual uinput device creation need root/input group
    // and are marked as integration tests

    #[tokio::test]
    #[ignore = "Requires uinput permissions"]
    async fn test_virtual_device_initialization() {
        let mut backend = WaylandInputBackend::new().await.unwrap();
        let result = backend.initialize().await;

        if result.is_ok() {
            assert!(backend.is_initialized());
            backend.cleanup().await.unwrap();
            assert!(!backend.is_initialized());
        }
        // If it fails, it's expected on systems without uinput permissions
    }
}
