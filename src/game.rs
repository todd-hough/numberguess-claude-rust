use rand::Rng;
use std::cmp::Ordering;

pub struct GuessingGame {
    min: i32,
    max: i32,
    secret_number: i32,
    guess_count: u32,
}

impl GuessingGame {
    pub fn new(min: i32, max: i32) -> Result<Self, String> {
        if max < min {
            return Err(format!("Maximum ({}) must be greater than or equal to minimum ({})", max, min));
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