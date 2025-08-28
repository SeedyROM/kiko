//! Reusable UI components for the Kiko frontend.
//!
//! Contains Yew components for session management, WebSocket communication,
//! and common UI elements used throughout the application.

pub mod confetti;
pub mod copy_url_button;
pub mod sessions;
pub mod websocket_chat;

pub use confetti::*;
pub use copy_url_button::*;
pub use sessions::*;
