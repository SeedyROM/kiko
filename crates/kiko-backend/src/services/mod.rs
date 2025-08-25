//! Backend services for session and participant management.
//!
//! This module provides the service layer abstractions and implementations
//! for managing sessions and their participants. Currently includes an
//! in-memory implementation suitable for development and testing.

pub mod sessions;

pub use sessions::*;
