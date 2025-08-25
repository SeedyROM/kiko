//! ID generation utilities for the Kiko application.
//!
//! This module provides type-safe ID generation using the `tiny_id` crate,
//! with specific ID types for different entities in the system.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::LazyLock;
use std::sync::Mutex;
use tiny_id::ShortCodeGenerator;

/// Type alias for a lazy-initialized short code generator with a mutex for thread safety.
/// This allows us to create a global generator that can be used across the application
/// without needing to pass it around explicitly.
type LazyShortCodeGenerator = LazyLock<Mutex<ShortCodeGenerator<char>>>;

// Global generator instances for different ID types
// Note: tiny_id generators need mutable access, so we wrap in Mutex
static SESSION_ID_GENERATOR: LazyShortCodeGenerator = LazyLock::new(|| {
    // Use alphanumeric but exclude confusing characters
    let alphabet: Vec<char> = "123456789ABCDEFGHJKMNPQRSTUVWXYZabcdefghkmnpqrstuvwxyz"
        .chars()
        .collect();
    Mutex::new(ShortCodeGenerator::with_alphabet(alphabet, 8))
});

static DEFAULT_ID_GENERATOR: LazyShortCodeGenerator = LazyLock::new(|| {
    let alphabet: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
        .chars()
        .collect();
    Mutex::new(ShortCodeGenerator::with_alphabet(alphabet, 8))
});

static SHORT_ID_GENERATOR: LazyShortCodeGenerator = LazyLock::new(|| {
    let alphabet: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
        .chars()
        .collect();
    Mutex::new(ShortCodeGenerator::with_alphabet(alphabet, 6))
});

/// A type-safe wrapper around string IDs with configurable length and format.
///
/// This struct provides a consistent way to generate and handle IDs throughout
/// the application while maintaining type safety and preventing ID mixing.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Id<T> {
    value: String,
    _phantom: std::marker::PhantomData<T>,
}

// Custom serde implementation to serialize as just a string
impl<T> Serialize for Id<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Id<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self::from_string(value))
    }
}

impl<T> Id<T> {
    /// Creates a new ID with the given value.
    ///
    /// # Arguments
    /// * `value` - The string value for the ID
    ///
    /// # Example
    /// ```
    /// use kiko::id::{Id, SessionMarker};
    ///
    /// let session_id = Id::<SessionMarker>::from_string("abc123".to_string());
    /// assert_eq!(session_id.as_str(), "abc123");
    /// ```
    pub fn from_string(value: String) -> Self {
        Self {
            value,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Generates a new random ID with the default length (8 characters).
    ///
    /// # Example
    /// ```
    /// use kiko::id::{Id, SessionMarker};
    ///
    /// let session_id = Id::<SessionMarker>::generate();
    /// assert_eq!(session_id.as_str().len(), 8);
    /// ```
    pub fn generate() -> Self {
        let mut generator = DEFAULT_ID_GENERATOR.lock().unwrap();
        Self {
            value: generator.next_string(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Generates a new random ID with a custom character set.
    ///
    /// # Arguments
    /// * `length` - The desired length of the generated ID
    /// * `chars` - The character set to use for generation
    ///
    /// # Example
    /// ```
    /// use kiko::id::{Id, SessionMarker};
    ///
    /// // Generate a numeric-only ID
    /// let numeric_id = Id::<SessionMarker>::generate_custom(6, "0123456789");
    /// assert_eq!(numeric_id.as_str().len(), 6);
    /// assert!(numeric_id.as_str().chars().all(|c| c.is_ascii_digit()));
    /// ```
    pub fn generate_custom(length: usize, chars: &str) -> Self {
        let alphabet: Vec<char> = chars.chars().collect();
        let mut generator = ShortCodeGenerator::with_alphabet(alphabet, length);

        Self {
            value: generator.next_string(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Returns the string value of the ID.
    ///
    /// # Example
    /// ```
    /// use kiko::id::{Id, SessionMarker};
    ///
    /// let id = Id::<SessionMarker>::from_string("test123".to_string());
    /// assert_eq!(id.as_str(), "test123");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Consumes the ID and returns the inner string value.
    ///
    /// # Example
    /// ```
    /// use kiko::id::{Id, SessionMarker};
    ///
    /// let id = Id::<SessionMarker>::from_string("test123".to_string());
    /// let value = id.into_string();
    /// assert_eq!(value, "test123");
    /// ```
    pub fn into_string(self) -> String {
        self.value
    }
}

impl<T> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Id({})", self.value)
    }
}

impl<T> fmt::Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<T> From<String> for Id<T> {
    fn from(value: String) -> Self {
        Self::from_string(value)
    }
}

impl<T> From<&str> for Id<T> {
    fn from(value: &str) -> Self {
        Self::from_string(value.to_string())
    }
}

impl<T> AsRef<str> for Id<T> {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

// Type markers for different entity types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionMarker;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticipantMarker;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StoryMarker;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VoteMarker;

/// Type alias for Session IDs
pub type SessionId = Id<SessionMarker>;

/// Type alias for Participant IDs  
pub type ParticipantId = Id<ParticipantMarker>;

/// Type alias for Story IDs
pub type StoryId = Id<StoryMarker>;

/// Type alias for Vote IDs
pub type VoteId = Id<VoteMarker>;

/// Convenience functions for generating common ID types
impl SessionId {
    /// Generates a new session ID with a user-friendly format (8 characters, mixed case).
    ///
    /// # Example
    /// ```
    /// use kiko::id::SessionId;
    ///
    /// let session_id = SessionId::new();
    /// assert_eq!(session_id.as_str().len(), 8);
    /// // Should not contain confusing characters like 0, O, I, l
    /// let confusing_chars = "0OIl";
    /// assert!(!session_id.as_str().chars().any(|c| confusing_chars.contains(c)));
    /// ```
    pub fn new() -> Self {
        let mut generator = SESSION_ID_GENERATOR.lock().unwrap();
        Self {
            value: generator.next_string(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl ParticipantId {
    /// Generates a new participant ID (6 characters, shorter for internal use).
    pub fn new() -> Self {
        let mut generator = SHORT_ID_GENERATOR.lock().unwrap();
        Self {
            value: generator.next_string(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl Default for ParticipantId {
    fn default() -> Self {
        Self::new()
    }
}

impl StoryId {
    /// Generates a new story ID (6 characters).
    pub fn new() -> Self {
        let mut generator = SHORT_ID_GENERATOR.lock().unwrap();
        Self {
            value: generator.next_string(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl VoteId {
    /// Generates a new vote ID (8 characters).
    pub fn new() -> Self {
        let mut generator = DEFAULT_ID_GENERATOR.lock().unwrap();
        Self {
            value: generator.next_string(),
            _phantom: std::marker::PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_generation() {
        let id1 = SessionId::generate();
        let id2 = SessionId::generate();

        // IDs should be different
        assert_ne!(id1, id2);

        // IDs should have correct length
        assert_eq!(id1.as_str().len(), 8);
        assert_eq!(id2.as_str().len(), 8);
    }

    #[test]
    fn test_id_creation() {
        let id = SessionId::from_string("test123".to_string());
        assert_eq!(id.as_str(), "test123");
        assert_eq!(id.to_string(), "test123");
    }

    #[test]
    fn test_id_from_string() {
        let id: SessionId = "abc123".into();
        assert_eq!(id.as_str(), "abc123");
    }

    #[test]
    fn test_custom_generation() {
        let alphabet: Vec<char> = "ABCD".chars().collect();
        let mut generator = ShortCodeGenerator::with_alphabet(alphabet, 4);
        let id = SessionId::from_string(generator.next_string());
        assert_eq!(id.as_str().len(), 4);
        assert!(id.as_str().chars().all(|c| "ABCD".contains(c)));
    }

    #[test]
    fn test_type_safety() {
        let session_id = SessionId::new();
        let participant_id = ParticipantId::new();

        // This should compile - same ID type
        let _same_session: SessionId = session_id.clone();

        // This would not compile - different ID types
        // let _wrong_type: ParticipantId = session_id;

        // Avoid unused variable warning
        let _used = participant_id.as_str();
    }

    #[test]
    fn test_session_id_format() {
        let id = SessionId::new();
        assert_eq!(id.as_str().len(), 8);

        // Should not contain confusing characters
        let confusing_chars = "0OIl";
        assert!(!id.as_str().chars().any(|c| confusing_chars.contains(c)));
    }

    #[test]
    fn test_serde() {
        let original = SessionId::from_string("test123".to_string());

        // Test serialization
        let serialized = serde_json::to_string(&original).unwrap();
        assert_eq!(serialized, "\"test123\"");

        // Test deserialization
        let deserialized: SessionId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }
}
