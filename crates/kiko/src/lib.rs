//! The shared library for Kiko, a Rust-based full-stack web application.
//!
//! This library provides the core functionality for both the frontend and backend
//! of the Kiko application, including API definitions, data structures, error handling, logging, and macros.

pub mod api;
pub mod data;
pub mod errors;
pub mod log;
pub mod macros;

pub use serde;
pub use serde_json;
pub use tracing;
