//! RemoteDesktop Session Manager
//!
//! Manages VNC server lifecycle and session state.
//!
//! ## Session Lifecycle
//!
//! ```text
//! Idle
//!   ↓ (request packet)
//! Starting
//!   ↓ (VNC server started)
//! Active
//!   ↓ (pause control)
//! Paused
//!   ↓ (resume control)
//! Active
//!   ↓ (stop control or error)
//! Stopped
//! ```

#[cfg(feature = "remotedesktop")]
use super::{
    capture::WaylandCapture,
    vnc::{generate_password, VncServer},
};
use crate::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

/// Session state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// No active session
    Idle,
    /// Session starting (VNC server initializing)
    Starting,
    /// Session active (VNC server running, client may or may not be connected)
    Active,
    /// Session paused
    Paused,
    /// Session stopped (cleanup in progress)
    Stopped,
    /// Session error
    Error,
}

/// Session information
#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// VNC server port
    pub port: u16,

    /// VNC password
    pub password: String,

    /// Framebuffer width
    pub width: u32,

    /// Framebuffer height
    pub height: u32,

    /// Current session state
    pub state: SessionState,
}

/// Session manager for VNC server
#[cfg(feature = "remotedesktop")]
pub struct SessionManager {
    /// Current session state
    state: Arc<RwLock<SessionState>>,

    /// Session information (if active)
    info: Arc<RwLock<Option<SessionInfo>>>,

    /// VNC server task handle
    server_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
}

#[cfg(feature = "remotedesktop")]
impl SessionManager {
    /// Create new session manager
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(SessionState::Idle)),
            info: Arc::new(RwLock::new(None)),
            server_handle: Arc::new(RwLock::new(None)),
        }
    }

    /// Get current session state
    pub async fn state(&self) -> SessionState {
        *self.state.read().await
    }

    /// Get session information
    pub async fn info(&self) -> Option<SessionInfo> {
        self.info.read().await.clone()
    }

    /// Start new VNC session
    ///
    /// # Arguments
    ///
    /// * `port` - VNC server port (typically 5900)
    /// * `quality` - Quality preset
    ///
    /// # Returns
    ///
    /// Session information with connection details
    pub async fn start_session(&mut self, port: u16) -> Result<SessionInfo> {
        let current_state = *self.state.read().await;
        if current_state != SessionState::Idle {
            return Err(crate::ProtocolError::invalid_state(format!(
                "Cannot start session in state: {:?}",
                current_state
            )));
        }

        info!("Starting RemoteDesktop session on port {}", port);
        *self.state.write().await = SessionState::Starting;

        // Create screen capture
        info!("Initializing Wayland screen capture...");
        let mut capture = WaylandCapture::new().await?;

        // Enumerate and select monitors
        let monitors = capture.enumerate_monitors().await?;
        if monitors.is_empty() {
            error!("No monitors found");
            *self.state.write().await = SessionState::Error;
            return Err(crate::ProtocolError::Plugin(
                "No monitors available".to_string(),
            ));
        }

        info!("Found {} monitor(s)", monitors.len());
        let primary_monitor = monitors.first().unwrap();
        info!(
            "Using monitor: {} ({}x{})",
            primary_monitor.name, primary_monitor.width, primary_monitor.height
        );

        let width = primary_monitor.width;
        let height = primary_monitor.height;

        // Select monitor(s)
        capture.select_monitors(vec![primary_monitor.id.clone()]);

        // Request screen capture permission
        info!("Requesting screen capture permission...");
        capture.request_permission().await?;

        // Start capture session
        capture.start_capture().await?;
        info!("Screen capture session started");

        // Generate VNC password
        let password = generate_password();
        debug!("Generated VNC password: {}", password);

        // Create session info
        let session_info = SessionInfo {
            port,
            password: password.clone(),
            width,
            height,
            state: SessionState::Active,
        };

        // Store session info
        *self.info.write().await = Some(session_info.clone());

        // Create and start VNC server in background task
        let state_clone = self.state.clone();
        let info_clone = self.info.clone();

        let server_handle = tokio::spawn(async move {
            info!("VNC server task starting...");

            // Create VNC server
            let mut server = VncServer::new(port, password);

            // Update state to active
            *state_clone.write().await = SessionState::Active;

            // Start VNC server (blocks until client disconnects)
            match server.start(capture).await {
                Ok(_) => {
                    info!("VNC server task completed normally");
                }
                Err(e) => {
                    error!("VNC server error: {}", e);
                    *state_clone.write().await = SessionState::Error;
                }
            }

            // Cleanup
            *state_clone.write().await = SessionState::Stopped;
            *info_clone.write().await = None;
        });

        *self.server_handle.write().await = Some(server_handle);

        info!("VNC session started successfully");
        Ok(session_info)
    }

    /// Stop current VNC session
    pub async fn stop_session(&mut self) -> Result<()> {
        let current_state = *self.state.read().await;
        if current_state == SessionState::Idle || current_state == SessionState::Stopped {
            debug!("No active session to stop");
            return Ok(());
        }

        info!("Stopping RemoteDesktop session");
        *self.state.write().await = SessionState::Stopped;

        // Abort VNC server task
        if let Some(handle) = self.server_handle.write().await.take() {
            handle.abort();
            debug!("VNC server task aborted");
        }

        // Clear session info
        *self.info.write().await = None;

        info!("RemoteDesktop session stopped");
        Ok(())
    }

    /// Pause current session (not fully supported yet)
    pub async fn pause_session(&mut self) -> Result<()> {
        let current_state = *self.state.read().await;
        if current_state != SessionState::Active {
            return Err(crate::ProtocolError::invalid_state(format!(
                "Cannot pause session in state: {:?}",
                current_state
            )));
        }

        warn!("Session pause not yet fully implemented");
        *self.state.write().await = SessionState::Paused;

        Ok(())
    }

    /// Resume paused session (not fully supported yet)
    pub async fn resume_session(&mut self) -> Result<()> {
        let current_state = *self.state.read().await;
        if current_state != SessionState::Paused {
            return Err(crate::ProtocolError::invalid_state(format!(
                "Cannot resume session in state: {:?}",
                current_state
            )));
        }

        warn!("Session resume not yet fully implemented");
        *self.state.write().await = SessionState::Active;

        Ok(())
    }
}

#[cfg(not(feature = "remotedesktop"))]
pub struct SessionManager;

#[cfg(not(feature = "remotedesktop"))]
impl SessionManager {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(feature = "remotedesktop")]
    async fn test_session_manager_creation() {
        let manager = SessionManager::new();
        assert_eq!(manager.state().await, SessionState::Idle);
        assert!(manager.info().await.is_none());
    }

    #[tokio::test]
    #[cfg(feature = "remotedesktop")]
    async fn test_session_state_transitions() {
        let mut manager = SessionManager::new();

        // Initial state
        assert_eq!(manager.state().await, SessionState::Idle);

        // Cannot pause from idle
        assert!(manager.pause_session().await.is_err());

        // Cannot resume from idle
        assert!(manager.resume_session().await.is_err());
    }
}
