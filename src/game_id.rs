//! Type-safe game ID wrapper.
//!
//! Provides a newtype pattern for game IDs to prevent accidentally
//! mixing u64 values and improve type safety.

use rand::Rng;
use serde::{Deserialize, Serialize};

/// A type-safe game identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GameId(u64);

impl GameId {
    /// Generate a new random game ID.
    pub fn new() -> Self {
        Self(rand::rng().random())
    }

    /// Create a GameId from a u64 value.
    pub fn from_u64(id: u64) -> Self {
        Self(id)
    }

    /// Get the inner u64 value.
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    /// Convert to i64 with proper error handling.
    pub fn to_i64(&self) -> Result<i64, String> {
        self.0
            .try_into()
            .map_err(|_| "Game ID exceeds i64 range".to_string())
    }
}

impl Default for GameId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for GameId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for GameId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<GameId> for u64 {
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
    fn test_game_id_from_u64() {
        let id = GameId::from_u64(12345);
        assert_eq!(id.as_u64(), 12345);
    }

    #[test]
    fn test_game_id_to_i64() {
        let id = GameId::from_u64(12345);
        assert_eq!(id.to_i64().unwrap(), 12345i64);
    }

    #[test]
    fn test_game_id_display() {
        let id = GameId::from_u64(12345);
        assert_eq!(format!("{}", id), "12345");
    }

    #[test]
    fn test_game_id_serialization() {
        let id = GameId::from_u64(12345);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "12345");

        let deserialized: GameId = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, id);
    }
}
