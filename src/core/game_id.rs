//! Type-safe game ID wrapper.
//!
//! Provides a newtype pattern for game IDs to prevent accidentally
//! mixing i64 values and improve type safety.

use rand::Rng;
use serde::{Deserialize, Serialize};

/// A type-safe game identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GameId(i64);

impl GameId {
    /// Generate a new random game ID.
    /// Generates a positive i64 value (0 to i64::MAX).
    // No Default impl on purpose: Default conventionally means a canonical,
    // predictable value, and new() is randomized. A previous randomized
    // Default was removed as misleading.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(rand::rng().random_range(0..i64::MAX))
    }
}

impl std::fmt::Display for GameId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i64> for GameId {
    fn from(id: i64) -> Self {
        Self(id)
    }
}

impl From<GameId> for i64 {
    fn from(id: GameId) -> Self {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_id_creation() {
        let id1 = GameId::new();
        let id2 = GameId::new();
        // IDs should be different (statistically)
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_game_id_from_i64() {
        let id = GameId::from(12345);
        assert_eq!(i64::from(id), 12345);
    }

    #[test]
    fn test_game_id_as_i64() {
        let id = GameId::from(12345);
        assert_eq!(i64::from(id), 12345i64);
    }

    #[test]
    fn test_game_id_display() {
        let id = GameId::from(12345);
        assert_eq!(format!("{id}"), "12345");
    }

    #[test]
    fn test_game_id_serialization() {
        let id = GameId::from(12345);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "12345");

        let deserialized: GameId = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, id);
    }
}
