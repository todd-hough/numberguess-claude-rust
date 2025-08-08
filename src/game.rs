use rand::Rng;
use std::cmp::Ordering;

const MAX_ALLOWED: i32 = 1_000_000;

pub struct GuessingGame {
    min: i32,
    max: i32,
    secret_number: i32,
    guess_count: u32,
}

impl GuessingGame {
    pub fn new(min: i32, max: i32) -> Result<Self, String> {
        // Validate that min and max are non-negative
        if min < 0 {
            return Err(format!("Minimum value ({}) must be non-negative (>= 0)", min));
        }
        
        if max < 0 {
            return Err(format!("Maximum value ({}) must be non-negative (>= 0)", max));
        }
        
        // Validate that max >= min
        if max < min {
            return Err(format!("Maximum ({}) must be greater than or equal to minimum ({})", max, min));
        }
        
        // Validate that values don't exceed reasonable limits
        if min > MAX_ALLOWED {
            return Err(format!("Minimum value ({}) exceeds maximum allowed value ({})", min, MAX_ALLOWED));
        }
        
        if max > MAX_ALLOWED {
            return Err(format!("Maximum value ({}) exceeds maximum allowed value ({})", max, MAX_ALLOWED));
        }
        
        // Check for potential overflow in range calculation
        // This is extra safety even though we limit to MAX_ALLOWED
        if max.saturating_sub(min) == i32::MAX {
            return Err(format!("Range between min ({}) and max ({}) is too large", min, max));
        }
        
        let secret_number = rand::thread_rng().gen_range(min..=max);
        
        Ok(GuessingGame {
            min,
            max,
            secret_number,
            guess_count: 0,
        })
    }
    
    pub fn get_range(&self) -> (i32, i32) {
        (self.min, self.max)
    }
    
    pub fn get_guess_count(&self) -> u32 {
        self.guess_count
    }
    
    pub fn make_guess(&mut self, guess: i32) -> GuessResult {
        self.guess_count += 1;
        
        match guess.cmp(&self.secret_number) {
            Ordering::Less => GuessResult::TooLow,
            Ordering::Greater => GuessResult::TooHigh,
            Ordering::Equal => GuessResult::Correct {
                number: self.secret_number,
                attempts: self.guess_count,
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum GuessResult {
    TooLow,
    TooHigh,
    Correct { number: i32, attempts: u32 },
}

impl GuessResult {
    pub fn is_correct(&self) -> bool {
        matches!(self, GuessResult::Correct { .. })
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
            assert!(e.contains("must be non-negative"));
        }
    }
    
    #[test]
    fn test_negative_max() {
        let game = GuessingGame::new(0, -10);
        assert!(game.is_err());
        if let Err(e) = game {
            assert!(e.contains("must be non-negative"));
        }
    }
    
    #[test]
    fn test_zero_values_allowed() {
        let game = GuessingGame::new(0, 0);
        assert!(game.is_ok());
        let game = game.unwrap();
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
            assert!(e.contains("exceeds maximum allowed value"));
        }
        
        // Test that exceeding MAX_ALLOWED for max is rejected
        let game = GuessingGame::new(0, MAX_ALLOWED + 1);
        assert!(game.is_err());
        if let Err(e) = game {
            assert!(e.contains("exceeds maximum allowed value"));
        }
    }
    
    #[test]
    fn test_large_valid_range() {
        // Test a large but valid range
        let game = GuessingGame::new(0, MAX_ALLOWED);
        assert!(game.is_ok());
        let game = game.unwrap();
        assert_eq!(game.get_range(), (0, MAX_ALLOWED));
        assert!(game.secret_number >= 0 && game.secret_number <= MAX_ALLOWED);
    }
    
    #[test]
    fn test_guess_result() {
        let mut game = GuessingGame::new(1, 10).unwrap();
        game.secret_number = 5;
        
        assert_eq!(game.make_guess(3), GuessResult::TooLow);
        assert_eq!(game.make_guess(7), GuessResult::TooHigh);
        assert_eq!(game.make_guess(5), GuessResult::Correct { number: 5, attempts: 3 });
    }
    
    #[test]
    fn test_guess_count() {
        let mut game = GuessingGame::new(1, 10).unwrap();
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
        assert!(GuessResult::Correct { number: 5, attempts: 3 }.is_correct());
    }
}