//! Difficulty calculation and classification feature.
//!
//! This module provides functionality to calculate the optimal number of guesses
//! needed for a number guessing game using binary search strategy, and to classify
//! the difficulty of a game based on the relationship between the guess limit
//! and the optimal number of guesses.
//!
//! # Architecture
//!
//! This module follows a clean separation of concerns:
//! - Pure calculation logic (no I/O, no web dependencies)
//! - Type-safe difficulty levels and information
//! - Fully unit tested
//!
//! # Examples
//!
//! ```
//! use number_guessing_game::features::difficulty::{calculate_difficulty, DifficultyLevel};
//!
//! // Calculate difficulty for a medium game (1-100, 10 guess limit)
//! let info = calculate_difficulty(1, 100, Some(10));
//! assert_eq!(info.optimal_guesses, 7);
//! assert_eq!(info.buffer, 3);
//! assert_eq!(info.level, DifficultyLevel::Easy);
//!
//! // Unlimited difficulty (no guess limit)
//! let info = calculate_difficulty(1, 100, None);
//! assert_eq!(info.level, DifficultyLevel::Unlimited);
//! ```

mod calculator;
mod types;

// Re-export public API
pub use calculator::calculate_optimal_guesses;
pub use types::{DifficultyInfo, DifficultyLevel, calculate_difficulty};
