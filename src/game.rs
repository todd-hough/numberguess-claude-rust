use rand::Rng;
use std::cmp::Ordering;
use thiserror::Error;

const MAX_ALLOWED: i32 = 1_000_000;

#[derive(Error, Debug)]
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
}

/// Validates that min and max values are within acceptable ranges
fn validate_range(min: i32, max: i32) -> Result<(), GameError> {
    if min < 0 {
        return Err(GameError::NegativeMin(min));
    }
    if max < 0 {
        return Err(GameError::NegativeMax(max));
    }
    if max < min {
        return Err(GameError::InvalidRange { min, max });
    }
    if min > MAX_ALLOWED {
        return Err(GameError::MinExceedsLimit {
            value: min,
            limit: MAX_ALLOWED,
        });
    }
    if max > MAX_ALLOWED {
        return Err(GameError::MaxExceedsLimit {
            value: max,
            limit: MAX_ALLOWED,
        });
    }
    if max.saturating_sub(min) == i32::MAX {
        return Err(GameError::RangeTooLarge { min, max });
    }
    Ok(())
}

pub struct GuessingGame {
    min: i32,
    max: i32,
    secret_number: i32,
    guess_count: u32,
    max_guesses: Option<u32>,
}

impl GuessingGame {
    pub fn new(min: i32, max: i32) -> Result<Self, GameError> {
        Self::new_with_limit(min, max, None)
    }

    pub fn new_with_limit(min: i32, max: i32, max_guesses: Option<u32>) -> Result<Self, GameError> {
        // Validate range
        validate_range(min, max)?;

        let secret_number = rand::rng().random_range(min..=max);

        Ok(GuessingGame {
            min,
            max,
            secret_number,
            guess_count: 0,
            max_guesses,
        })
    }

    pub fn get_range(&self) -> (i32, i32) {
        (self.min, self.max)
    }

    pub fn get_guess_count(&self) -> u32 {
        self.guess_count
    }

    pub fn get_max_guesses(&self) -> Option<u32> {
        self.max_guesses
    }

    pub fn secret_number(&self) -> i32 {
        self.secret_number
    }

    pub fn from_db(
        min: i32,
        max: i32,
        secret_number: i32,
        guess_count: u32,
        max_guesses: Option<u32>,
    ) -> Result<Self, GameError> {
        // Validate range
        validate_range(min, max)?;

        // Validate secret number is in range
        if secret_number < min || secret_number > max {
            return Err(GameError::SecretOutOfRange {
                secret: secret_number,
                min,
                max,
            });
        }

        Ok(GuessingGame {
            min,
            max,
            secret_number,
            guess_count,
            max_guesses,
        })
    }

    pub fn has_guesses_remaining(&self) -> bool {
        match self.max_guesses {
            Some(max) => self.guess_count < max,
            None => true,
        }
    }

    #[cfg(test)]
    pub fn set_secret_for_testing(&mut self, secret: i32) {
        self.secret_number = secret;
    }

    pub fn make_guess(&mut self, guess: i32) -> GuessResult {
        // Check if guess limit has been reached before this guess
        if !self.has_guesses_remaining() {
            return GuessResult::LimitReached {
                number: self.secret_number,
                max_guesses: self.max_guesses.unwrap_or(0),
            };
        }

        self.guess_count += 1;

        let result = match guess.cmp(&self.secret_number) {
            Ordering::Less => GuessResult::TooLow,
            Ordering::Greater => GuessResult::TooHigh,
            Ordering::Equal => GuessResult::Correct {
                number: self.secret_number,
                attempts: self.guess_count,
            },
        };

        // Check if this was the last allowed guess and it wasn't correct
        if !result.is_correct() && !self.has_guesses_remaining() {
            return GuessResult::LimitReached {
                number: self.secret_number,
                max_guesses: self.max_guesses.unwrap_or(0),
            };
        }

        result
    }
}

#[derive(Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_creation() {
        let game = GuessingGame::new(1, 10);
        assert!(game.is_ok());

        let game = game.unwrap();
        assert_eq!(game.get_range(), (1, 10));
    }

    #[test]
    fn test_invalid_range() {
        let game = GuessingGame::new(10, 1);
        assert!(game.is_err());
    }

    #[test]
    fn test_negative_min() {
        let game = GuessingGame::new(-5, 10);
        assert!(game.is_err());
        if let Err(e) = game {
            assert!(e.to_string().contains("must be non-negative"));
        }
    }

    #[test]
    fn test_negative_max() {
        let game = GuessingGame::new(0, -10);
        assert!(game.is_err());
        if let Err(e) = game {
            assert!(e.to_string().contains("must be non-negative"));
        }
    }

    #[test]
    fn test_zero_values_allowed() {
        let game = GuessingGame::new(0, 0);
        assert!(game.is_ok());
        let game = game.expect("Should create game with min=0 and max=0");
        assert_eq!(game.get_range(), (0, 0));
        assert_eq!(game.secret_number, 0);
    }

    #[test]
    fn test_max_allowed_limit() {
        // Test that MAX_ALLOWED is accepted
        let game = GuessingGame::new(0, MAX_ALLOWED);
        assert!(game.is_ok());

        // Test that exceeding MAX_ALLOWED for min is rejected
        let game = GuessingGame::new(MAX_ALLOWED + 1, MAX_ALLOWED + 2);
        assert!(game.is_err());
        if let Err(e) = game {
            assert!(e.to_string().contains("exceeds maximum allowed value"));
        }

        // Test that exceeding MAX_ALLOWED for max is rejected
        let game = GuessingGame::new(0, MAX_ALLOWED + 1);
        assert!(game.is_err());
        if let Err(e) = game {
            assert!(e.to_string().contains("exceeds maximum allowed value"));
        }
    }

    #[test]
    fn test_large_valid_range() {
        // Test a large but valid range
        let game = GuessingGame::new(0, MAX_ALLOWED);
        assert!(game.is_ok());
        let game = game.expect("Should create game with maximum allowed range");
        assert_eq!(game.get_range(), (0, MAX_ALLOWED));
        assert!(game.secret_number >= 0 && game.secret_number <= MAX_ALLOWED);
    }

    #[test]
    fn test_guess_result() {
        let mut game = GuessingGame::new(1, 10).expect("Should create game with range 1-10");
        game.set_secret_for_testing(5);

        assert_eq!(game.make_guess(3), GuessResult::TooLow);
        assert_eq!(game.make_guess(7), GuessResult::TooHigh);
        assert_eq!(
            game.make_guess(5),
            GuessResult::Correct {
                number: 5,
                attempts: 3
            }
        );
    }

    #[test]
    fn test_guess_count() {
        let mut game = GuessingGame::new(1, 10).expect("Should create game with range 1-10");
        assert_eq!(game.get_guess_count(), 0);

        game.make_guess(5);
        assert_eq!(game.get_guess_count(), 1);

        game.make_guess(3);
        assert_eq!(game.get_guess_count(), 2);
    }

    #[test]
    fn test_is_correct() {
        assert!(!GuessResult::TooLow.is_correct());
        assert!(!GuessResult::TooHigh.is_correct());
        assert!(
            GuessResult::Correct {
                number: 5,
                attempts: 3
            }
            .is_correct()
        );
        assert!(
            !GuessResult::LimitReached {
                number: 5,
                max_guesses: 10
            }
            .is_correct()
        );
    }

    #[test]
    fn test_is_game_over() {
        assert!(!GuessResult::TooLow.is_game_over());
        assert!(!GuessResult::TooHigh.is_game_over());
        assert!(
            GuessResult::Correct {
                number: 5,
                attempts: 3
            }
            .is_game_over()
        );
        assert!(
            GuessResult::LimitReached {
                number: 5,
                max_guesses: 10
            }
            .is_game_over()
        );
    }

    #[test]
    fn test_game_with_guess_limit() {
        let mut game = GuessingGame::new_with_limit(1, 10, Some(3)).expect("Should create game with guess limit");
        game.set_secret_for_testing(5);

        assert_eq!(game.get_max_guesses(), Some(3));
        assert!(game.has_guesses_remaining());

        // First guess
        assert_eq!(game.make_guess(1), GuessResult::TooLow);
        assert!(game.has_guesses_remaining());

        // Second guess
        assert_eq!(game.make_guess(10), GuessResult::TooHigh);
        assert!(game.has_guesses_remaining());

        // Third guess (final)
        assert_eq!(
            game.make_guess(3),
            GuessResult::LimitReached {
                number: 5,
                max_guesses: 3
            }
        );
        assert!(!game.has_guesses_remaining());

        // Attempt after limit should return LimitReached immediately
        assert_eq!(
            game.make_guess(5),
            GuessResult::LimitReached {
                number: 5,
                max_guesses: 3
            }
        );
    }

    #[test]
    fn test_game_with_no_limit() {
        let mut game = GuessingGame::new_with_limit(1, 10, None).expect("Should create game with no guess limit");
        game.set_secret_for_testing(5);

        assert_eq!(game.get_max_guesses(), None);

        // Many guesses should be allowed
        let mut guess_count = 0;
        for i in 1..20 {
            assert!(game.has_guesses_remaining());
            if i != 5 {
                let result = game.make_guess(i);
                assert!(!result.is_game_over());
                guess_count += 1;
            }
        }

        // Finally guess correctly
        assert_eq!(
            game.make_guess(5),
            GuessResult::Correct {
                number: 5,
                attempts: guess_count + 1
            }
        );
    }

    #[test]
    fn test_correct_guess_within_limit() {
        let mut game = GuessingGame::new_with_limit(1, 10, Some(5)).expect("Should create game with range 1-10 and limit 5");
        game.set_secret_for_testing(7);

        assert_eq!(game.make_guess(3), GuessResult::TooLow);
        assert_eq!(game.make_guess(9), GuessResult::TooHigh);
        assert_eq!(
            game.make_guess(7),
            GuessResult::Correct {
                number: 7,
                attempts: 3
            }
        );
    }
}
