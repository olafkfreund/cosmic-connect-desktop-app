//! Connection Management
//!
//! This module provides TLS connection management for secure communication
//! between paired devices.

pub mod events;
pub mod manager;

pub use events::ConnectionEvent;
pub use manager::{ConnectionConfig, ConnectionManager};
