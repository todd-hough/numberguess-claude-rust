use rand::Rng;
use std::cmp::Ordering;

use crate::core::validators::validate_range;
use crate::core::{GameError, GuessResult};

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
        // Validate range using shared validator
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

    pub fn range(&self) -> (i32, i32) {
        (self.min, self.max)
    }

    pub fn guess_count(&self) -> u32 {
        self.guess_count
    }

    pub fn max_guesses(&self) -> Option<u32> {
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
        // Validate range using shared validator
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::validators::MAX_RANGE;

    #[test]
    fn test_game_creation() {
        let game = GuessingGame::new(1, 10);
        assert!(game.is_ok());

        let game = game.unwrap();
        assert_eq!(game.range(), (1, 10));
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
        assert_eq!(game.range(), (0, 0));
        assert_eq!(game.secret_number, 0);
    }

    #[test]
    fn test_max_allowed_limit() {
        // Test that MAX_RANGE is accepted
        let game = GuessingGame::new(0, MAX_RANGE);
        assert!(game.is_ok());

        // Test that exceeding MAX_RANGE for min is rejected
        let game = GuessingGame::new(MAX_RANGE + 1, MAX_RANGE + 2);
        assert!(game.is_err());
        if let Err(e) = game {
            assert!(e.to_string().contains("exceeds maximum allowed value"));
        }

        // Test that exceeding MAX_RANGE for max is rejected
        let game = GuessingGame::new(0, MAX_RANGE + 1);
        assert!(game.is_err());
        if let Err(e) = game {
            assert!(e.to_string().contains("exceeds maximum allowed value"));
        }
    }

    #[test]
    fn test_large_valid_range() {
        // Test a large but valid range
        let game = GuessingGame::new(0, MAX_RANGE);
        assert!(game.is_ok());
        let game = game.expect("Should create game with maximum allowed range");
        assert_eq!(game.range(), (0, MAX_RANGE));
        assert!(game.secret_number >= 0 && game.secret_number <= MAX_RANGE);
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
        assert_eq!(game.guess_count(), 0);

        game.make_guess(5);
        assert_eq!(game.guess_count(), 1);

        game.make_guess(3);
        assert_eq!(game.guess_count(), 2);
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
        let mut game = GuessingGame::new_with_limit(1, 10, Some(3))
            .expect("Should create game with guess limit");
        game.set_secret_for_testing(5);

        assert_eq!(game.max_guesses(), Some(3));
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
        let mut game = GuessingGame::new_with_limit(1, 10, None)
            .expect("Should create game with no guess limit");
        game.set_secret_for_testing(5);

        assert_eq!(game.max_guesses(), None);

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
        let mut game = GuessingGame::new_with_limit(1, 10, Some(5))
            .expect("Should create game with range 1-10 and limit 5");
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

    #[test]
    fn test_from_db_with_secret_below_range() {
        // Secret number below min should be rejected
        let result = GuessingGame::from_db(10, 20, 5, 0, None);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, GameError::SecretOutOfRange { .. }));
            assert!(e.to_string().contains("Secret number (5)"));
            assert!(e.to_string().contains("between min (10) and max (20)"));
        }
    }

    #[test]
    fn test_from_db_with_secret_above_range() {
        // Secret number above max should be rejected
        let result = GuessingGame::from_db(10, 20, 25, 0, None);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, GameError::SecretOutOfRange { .. }));
            assert!(e.to_string().contains("Secret number (25)"));
        }
    }

    #[test]
    fn test_from_db_with_secret_at_min_boundary() {
        // Secret exactly at min should be valid
        let result = GuessingGame::from_db(10, 20, 10, 0, None);
        assert!(result.is_ok());
        let game = result.unwrap();
        assert_eq!(game.secret_number(), 10);
    }

    #[test]
    fn test_from_db_with_secret_at_max_boundary() {
        // Secret exactly at max should be valid
        let result = GuessingGame::from_db(10, 20, 20, 0, None);
        assert!(result.is_ok());
        let game = result.unwrap();
        assert_eq!(game.secret_number(), 20);
    }

    #[test]
    fn test_from_db_with_valid_secret() {
        // Valid secret within range
        let result = GuessingGame::from_db(1, 100, 50, 5, Some(10));
        assert!(result.is_ok());
        let game = result.unwrap();
        assert_eq!(game.range(), (1, 100));
        assert_eq!(game.secret_number(), 50);
        assert_eq!(game.guess_count(), 5);
        assert_eq!(game.max_guesses(), Some(10));
    }

    #[test]
    fn test_from_db_validates_range() {
        // from_db should still validate the range itself
        let result = GuessingGame::from_db(100, 10, 50, 0, None);
        assert!(result.is_err());
        // Should fail range validation (max < min) before checking secret
    }
}
