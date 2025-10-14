//! Type definitions for difficulty metrics.
//!
//! Defines the difficulty levels and difficulty information structure
//! with helper methods for display and classification.

use super::calculator::calculate_optimal_guesses;

/// Difficulty level classification based on buffer between guess limit and optimal guesses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DifficultyLevel {
    /// No guess limit - unlimited attempts
    Unlimited,
    /// Buffer >= 5 - very forgiving, great for beginners
    VeryEasy,
    /// Buffer 3-4 - comfortable challenge
    Easy,
    /// Buffer 2 - balanced difficulty
    Medium,
    /// Buffer 1 - challenging, requires good strategy
    Hard,
    /// Buffer 0 - perfect play required
    Expert,
    /// Buffer < 0 - below optimal, nearly impossible
    Impossible,
}

impl DifficultyLevel {
    /// Returns the display name of the difficulty level.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Unlimited => "UNLIMITED",
            Self::VeryEasy => "VERY EASY",
            Self::Easy => "EASY",
            Self::Medium => "MEDIUM",
            Self::Hard => "HARD",
            Self::Expert => "EXPERT",
            Self::Impossible => "IMPOSSIBLE",
        }
    }

    /// Returns the emoji icon for the difficulty level.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Unlimited => "♾️",
            Self::VeryEasy => "🌱",
            Self::Easy => "😊",
            Self::Medium => "🎯",
            Self::Hard => "🔥",
            Self::Expert => "⚡",
            Self::Impossible => "💀",
        }
    }

    /// Returns the CSS color for the difficulty level.
    pub fn color(&self) -> &'static str {
        match self {
            Self::Unlimited => "#6c757d",  // Gray
            Self::VeryEasy => "#28a745",   // Green
            Self::Easy => "#5cb85c",       // Light green
            Self::Medium => "#ffc107",     // Yellow/amber
            Self::Hard => "#fd7e14",       // Orange
            Self::Expert => "#dc3545",     // Red
            Self::Impossible => "#6f42c1", // Purple
        }
    }

    /// Returns the meter width percentage for visual display (0-100).
    pub fn meter_width(&self) -> u8 {
        match self {
            Self::Unlimited => 0,
            Self::VeryEasy => 30,
            Self::Easy => 45,
            Self::Medium => 60,
            Self::Hard => 75,
            Self::Expert => 90,
            Self::Impossible => 100,
        }
    }

    /// Returns the contextual message for the difficulty level.
    pub fn message(&self) -> &'static str {
        match self {
            Self::Unlimited => "Take your time! No guess limit means you can experiment freely.",
            Self::VeryEasy => "Great for beginners! You have plenty of room to learn.",
            Self::Easy => "A comfortable challenge with room for mistakes.",
            Self::Medium => "Good balance of challenge and fun! Use a smart strategy.",
            Self::Hard => "This is challenging! You'll need an efficient approach to win.",
            Self::Expert => "No room for error! Perfect binary search required.",
            Self::Impossible => {
                "Your limit is below optimal. You'd need perfect play plus incredible luck!"
            }
        }
    }

    /// Returns the CSS class for styling.
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Unlimited => "difficulty-unlimited",
            Self::VeryEasy => "difficulty-very-easy",
            Self::Easy => "difficulty-easy",
            Self::Medium => "difficulty-medium",
            Self::Hard => "difficulty-hard",
            Self::Expert => "difficulty-expert",
            Self::Impossible => "difficulty-impossible",
        }
    }
}

/// Complete difficulty information for a game configuration.
#[derive(Debug, Clone)]
pub struct DifficultyInfo {
    /// Minimum number in range
    pub min: i32,
    /// Maximum number in range
    pub max: i32,
    /// Total numbers in range (max - min + 1)
    pub range_size: u32,
    /// Optimal guesses using binary search
    pub optimal_guesses: u32,
    /// User's guess limit (None if unlimited)
    pub guess_limit: Option<u32>,
    /// Extra guesses beyond optimal (negative if impossible)
    pub buffer: i32,
    /// Classified difficulty level
    pub level: DifficultyLevel,
}

impl DifficultyInfo {
    /// Returns true if a guess limit is set.
    pub fn has_limit(&self) -> bool {
        self.guess_limit.is_some()
    }

    /// Returns a formatted description of the range size.
    pub fn range_size_description(&self) -> String {
        let number_word = if self.range_size == 1 {
            "number"
        } else {
            "numbers"
        };
        format!("{} {}", self.range_size, number_word)
    }

    /// Returns a formatted description of optimal guesses.
    pub fn optimal_description(&self) -> String {
        let guess_word = if self.optimal_guesses == 1 {
            "guess"
        } else {
            "guesses"
        };
        format!("{} {}", self.optimal_guesses, guess_word)
    }

    /// Returns a formatted description of the guess limit.
    pub fn limit_description(&self) -> String {
        match self.guess_limit {
            Some(limit) => {
                let guess_word = if limit == 1 { "guess" } else { "guesses" };
                format!("{} {}", limit, guess_word)
            }
            None => "unlimited".to_string(),
        }
    }

    /// Returns a formatted description of the buffer.
    pub fn buffer_description(&self) -> String {
        if self.buffer < 0 {
            let abs_buffer = self.buffer.abs();
            let guess_word = if abs_buffer == 1 { "guess" } else { "guesses" };
            format!("{} {} below optimal!", abs_buffer, guess_word)
        } else if self.buffer == 0 {
            "no extra guesses".to_string()
        } else {
            let guess_word = if self.buffer == 1 { "guess" } else { "guesses" };
            format!("{} extra {}", self.buffer, guess_word)
        }
    }
}

/// Calculates complete difficulty information for a game configuration.
///
/// # Arguments
/// * `min` - Minimum number in range
/// * `max` - Maximum number in range
/// * `guess_limit` - Optional guess limit (None for unlimited)
///
/// # Returns
/// Complete `DifficultyInfo` with calculated metrics and classification.
///
/// # Examples
/// ```
/// use number_guessing_game::features::difficulty::calculate_difficulty;
///
/// // Medium difficulty: 1-100 with 10 guess limit
/// let info = calculate_difficulty(1, 100, Some(10));
/// assert_eq!(info.optimal_guesses, 7);
/// assert_eq!(info.buffer, 3);
/// ```
pub fn calculate_difficulty(min: i32, max: i32, guess_limit: Option<u32>) -> DifficultyInfo {
    let range_size = (max - min + 1) as u32;
    let optimal_guesses = calculate_optimal_guesses(min, max);

    // Calculate buffer and classify difficulty
    let (buffer, level) = match guess_limit {
        None => (0, DifficultyLevel::Unlimited),
        Some(limit) => {
            let buffer = limit as i32 - optimal_guesses as i32;
            let level = classify_difficulty(buffer);
            (buffer, level)
        }
    };

    DifficultyInfo {
        min,
        max,
        range_size,
        optimal_guesses,
        guess_limit,
        buffer,
        level,
    }
}

/// Classifies difficulty level based on buffer.
fn classify_difficulty(buffer: i32) -> DifficultyLevel {
    match buffer {
        b if b < 0 => DifficultyLevel::Impossible,
        0 => DifficultyLevel::Expert,
        1 => DifficultyLevel::Hard,
        2 => DifficultyLevel::Medium,
        3..=4 => DifficultyLevel::Easy,
        _ => DifficultyLevel::VeryEasy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_level_classification() {
        // Test buffer-based classification
        assert_eq!(classify_difficulty(-5), DifficultyLevel::Impossible);
        assert_eq!(classify_difficulty(-1), DifficultyLevel::Impossible);
        assert_eq!(classify_difficulty(0), DifficultyLevel::Expert);
        assert_eq!(classify_difficulty(1), DifficultyLevel::Hard);
        assert_eq!(classify_difficulty(2), DifficultyLevel::Medium);
        assert_eq!(classify_difficulty(3), DifficultyLevel::Easy);
        assert_eq!(classify_difficulty(4), DifficultyLevel::Easy);
        assert_eq!(classify_difficulty(5), DifficultyLevel::VeryEasy);
        assert_eq!(classify_difficulty(10), DifficultyLevel::VeryEasy);
    }

    #[test]
    fn test_calculate_difficulty_unlimited() {
        let info = calculate_difficulty(1, 100, None);

        assert_eq!(info.min, 1);
        assert_eq!(info.max, 100);
        assert_eq!(info.range_size, 100);
        assert_eq!(info.optimal_guesses, 7);
        assert_eq!(info.guess_limit, None);
        assert_eq!(info.level, DifficultyLevel::Unlimited);
        assert!(!info.has_limit());
    }

    #[test]
    fn test_calculate_difficulty_very_easy() {
        let info = calculate_difficulty(1, 100, Some(12));

        assert_eq!(info.optimal_guesses, 7);
        assert_eq!(info.guess_limit, Some(12));
        assert_eq!(info.buffer, 5);
        assert_eq!(info.level, DifficultyLevel::VeryEasy);
        assert!(info.has_limit());
    }

    #[test]
    fn test_calculate_difficulty_easy() {
        let info = calculate_difficulty(1, 100, Some(10));

        assert_eq!(info.buffer, 3);
        assert_eq!(info.level, DifficultyLevel::Easy);
    }

    #[test]
    fn test_calculate_difficulty_medium() {
        let info = calculate_difficulty(1, 100, Some(9));

        assert_eq!(info.buffer, 2);
        assert_eq!(info.level, DifficultyLevel::Medium);
    }

    #[test]
    fn test_calculate_difficulty_hard() {
        let info = calculate_difficulty(1, 100, Some(8));

        assert_eq!(info.buffer, 1);
        assert_eq!(info.level, DifficultyLevel::Hard);
    }

    #[test]
    fn test_calculate_difficulty_expert() {
        let info = calculate_difficulty(1, 1000, Some(10));

        assert_eq!(info.optimal_guesses, 10);
        assert_eq!(info.buffer, 0);
        assert_eq!(info.level, DifficultyLevel::Expert);
    }

    #[test]
    fn test_calculate_difficulty_impossible() {
        let info = calculate_difficulty(1, 100, Some(6));

        assert_eq!(info.optimal_guesses, 7);
        assert_eq!(info.buffer, -1);
        assert_eq!(info.level, DifficultyLevel::Impossible);
    }

    #[test]
    fn test_difficulty_info_descriptions() {
        let info = calculate_difficulty(1, 100, Some(10));

        assert_eq!(info.range_size_description(), "100 numbers");
        assert_eq!(info.optimal_description(), "7 guesses");
        assert_eq!(info.limit_description(), "10 guesses");
        assert_eq!(info.buffer_description(), "3 extra guesses");
    }

    #[test]
    fn test_difficulty_info_descriptions_singular() {
        let info = calculate_difficulty(5, 5, Some(1));

        assert_eq!(info.range_size_description(), "1 number");
        assert_eq!(info.optimal_description(), "1 guess");
        assert_eq!(info.limit_description(), "1 guess");
    }

    #[test]
    fn test_difficulty_info_buffer_descriptions() {
        // Positive buffer
        let info = calculate_difficulty(1, 100, Some(10));
        assert_eq!(info.buffer_description(), "3 extra guesses");

        // Zero buffer
        let info = calculate_difficulty(1, 1000, Some(10));
        assert_eq!(info.buffer_description(), "no extra guesses");

        // Negative buffer
        let info = calculate_difficulty(1, 100, Some(5));
        assert_eq!(info.buffer_description(), "2 guesses below optimal!");

        // Single negative
        let info = calculate_difficulty(1, 100, Some(6));
        assert_eq!(info.buffer_description(), "1 guess below optimal!");
    }

    #[test]
    fn test_difficulty_level_methods() {
        let level = DifficultyLevel::Medium;

        assert_eq!(level.name(), "MEDIUM");
        assert_eq!(level.icon(), "🎯");
        assert_eq!(level.color(), "#ffc107");
        assert_eq!(level.meter_width(), 60);
        assert!(level.message().contains("balance"));
        assert_eq!(level.css_class(), "difficulty-medium");
    }

    #[test]
    fn test_all_difficulty_levels_have_valid_data() {
        let levels = vec![
            DifficultyLevel::Unlimited,
            DifficultyLevel::VeryEasy,
            DifficultyLevel::Easy,
            DifficultyLevel::Medium,
            DifficultyLevel::Hard,
            DifficultyLevel::Expert,
            DifficultyLevel::Impossible,
        ];

        for level in levels {
            // Ensure all methods return non-empty strings
            assert!(!level.name().is_empty());
            assert!(!level.icon().is_empty());
            assert!(!level.color().is_empty());
            assert!(!level.message().is_empty());
            assert!(!level.css_class().is_empty());

            // Meter width should be 0-100
            assert!(level.meter_width() <= 100);
        }
    }
}
