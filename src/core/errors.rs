use thiserror::Error;

/// Error types for game operations
#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum GameError {
    #[error("Minimum value ({0}) must be non-negative (>= 0)")]
    NegativeMin(i32),

    #[error("Maximum value ({0}) must be non-negative (>= 0)")]
    NegativeMax(i32),

    #[error("Maximum ({max}) must be greater than or equal to minimum ({min})")]
    InvalidRange { min: i32, max: i32 },

    #[error("Minimum value ({value}) exceeds maximum allowed value ({limit})")]
    MinExceedsLimit { value: i32, limit: i32 },

    #[error("Maximum value ({value}) exceeds maximum allowed value ({limit})")]
    MaxExceedsLimit { value: i32, limit: i32 },

    #[error("Range between min ({min}) and max ({max}) is too large")]
    RangeTooLarge { min: i32, max: i32 },

    #[error("Secret number ({secret}) must be between min ({min}) and max ({max})")]
    SecretOutOfRange { secret: i32, min: i32, max: i32 },

    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Results of a guess attempt
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum GuessResult {
    TooLow,
    TooHigh,
    Correct { number: i32, attempts: u32 },
    LimitReached { number: i32, max_guesses: u32 },
}

impl GuessResult {
    pub fn is_correct(&self) -> bool {
        matches!(self, GuessResult::Correct { .. })
    }

    pub fn is_game_over(&self) -> bool {
        matches!(
            self,
            GuessResult::Correct { .. } | GuessResult::LimitReached { .. }
        )
    }
}
