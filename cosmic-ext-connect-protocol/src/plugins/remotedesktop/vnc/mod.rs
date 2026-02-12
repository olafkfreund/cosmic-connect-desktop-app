//! VNC Server Implementation
//!
//! Provides VNC (Virtual Network Computing) server functionality for remote desktop access.
//!
//! ## Modules
//!
//! - `protocol`: RFB protocol constants and message types
//! - `auth`: VNC authentication (security type 2)
//! - `encoding`: Frame encoding with multiple compression types (Raw, LZ4, H.264, Hextile)
//! - `streaming`: Async streaming pipeline from screen capture to encoded frames
//! - `server`: VNC server with TCP listener and protocol implementation
//!
//! ## Architecture
//!
//! ```text
//! VNC Server (port 5900)
//!       ↓
//! RFB Handshake + Auth
//!       ↓
//! Protocol Loop
//!       ↓
//! Capture (30 FPS)
//!       ↓
//! [Raw Frame Queue]
//!       ↓
//! Encoder (Async)
//!       ↓
//! [Encoded Frame Queue]
//!       ↓
//! Framebuffer Updates
//! ```

pub mod auth;
pub mod encoding;
pub mod protocol;
pub mod server;
pub mod streaming;

pub use auth::{generate_password, VncAuth};
pub use encoding::{EncoderStats, FrameEncoder};
pub use protocol::{
    ClientMessage, FramebufferUpdate, FramebufferUpdateRequest, KeyEvent, PixelFormat,
    PointerEvent, Rectangle, RfbEncoding, ServerInit, ServerMessage,
};
pub use server::{ServerState, VncServer};
pub use streaming::{StreamConfig, StreamState, StreamStats, StreamingSession};
