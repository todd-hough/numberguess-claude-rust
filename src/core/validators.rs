//! Pure validation functions for game parameters.
//!
//! This module contains validation logic with no I/O dependencies,
//! used by both CLI and web interfaces.

use crate::core::GameError;

/// Maximum allowed value for min/max range
pub const MAX_RANGE: i32 = 1_000_000;

/// Maximum guess limit for web/API (lower for security)
pub const MAX_WEB_GUESS_LIMIT: u32 = 100;

/// Maximum guess limit for CLI
pub const MAX_CLI_GUESS_LIMIT: u32 = 1000;

/// Validates a minimum value
pub fn validate_min_value(min: i32) -> Result<(), GameError> {
    if min < 0 {
        return Err(GameError::NegativeMin(min));
    }
    if min > MAX_RANGE {
        return Err(GameError::MinExceedsLimit {
            value: min,
            limit: MAX_RANGE,
        });
    }
    Ok(())
}

/// Validates a maximum value
pub fn validate_max_value(max: i32) -> Result<(), GameError> {
    if max < 0 {
        return Err(GameError::NegativeMax(max));
    }
    if max > MAX_RANGE {
        return Err(GameError::MaxExceedsLimit {
            value: max,
            limit: MAX_RANGE,
        });
    }
    Ok(())
}

/// Validates that max >= min
pub fn validate_max_gte_min(min: i32, max: i32) -> Result<(), GameError> {
    if max < min {
        return Err(GameError::InvalidRange { min, max });
    }
    Ok(())
}

/// Validates both min and max values together
pub fn validate_range(min: i32, max: i32) -> Result<(), GameError> {
    validate_min_value(min)?;
    validate_max_value(max)?;
    validate_max_gte_min(min, max)?;

    if max.saturating_sub(min) == i32::MAX {
        return Err(GameError::RangeTooLarge { min, max });
    }

    Ok(())
}

/// Validates a guess limit and returns the adjusted limit (or None for no limit)
pub fn validate_guess_limit(limit: u32, max_limit: u32) -> Result<Option<u32>, GameError> {
    if limit == 0 {
        return Ok(None); // 0 means no limit
    }
    if limit > max_limit {
        return Err(GameError::ValidationError(format!(
            "Guess limit ({}) exceeds maximum allowed ({})",
            limit, max_limit
        )));
    }
    Ok(Some(limit))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_min_value() {
        assert!(validate_min_value(0).is_ok());
        assert!(validate_min_value(100).is_ok());
        assert!(validate_min_value(MAX_RANGE).is_ok());
        assert!(matches!(
            validate_min_value(-1),
            Err(GameError::NegativeMin(-1))
        ));
        assert!(matches!(
            validate_min_value(MAX_RANGE + 1),
            Err(GameError::MinExceedsLimit { .. })
        ));
    }

    #[test]
    fn test_validate_max_value() {
        assert!(validate_max_value(0).is_ok());
        assert!(validate_max_value(100).is_ok());
        assert!(validate_max_value(MAX_RANGE).is_ok());
        assert!(matches!(
            validate_max_value(-1),
            Err(GameError::NegativeMax(-1))
        ));
        assert!(matches!(
            validate_max_value(MAX_RANGE + 1),
            Err(GameError::MaxExceedsLimit { .. })
        ));
    }

    #[test]
    fn test_validate_max_gte_min() {
        assert!(validate_max_gte_min(0, 0).is_ok());
        assert!(validate_max_gte_min(0, 100).is_ok());
        assert!(matches!(
            validate_max_gte_min(100, 0),
            Err(GameError::InvalidRange { min: 100, max: 0 })
        ));
    }

    #[test]
    fn test_validate_range() {
        assert!(validate_range(0, 100).is_ok());
        assert!(validate_range(0, MAX_RANGE).is_ok());
        assert!(validate_range(-1, 100).is_err());
        assert!(validate_range(0, -1).is_err());
        assert!(validate_range(100, 50).is_err());
        assert!(validate_range(0, MAX_RANGE + 1).is_err());
    }

    #[test]
    fn test_validate_guess_limit() {
        assert_eq!(validate_guess_limit(0, 100).unwrap(), None);
        assert_eq!(validate_guess_limit(10, 100).unwrap(), Some(10));
        assert_eq!(validate_guess_limit(100, 100).unwrap(), Some(100));
        assert!(validate_guess_limit(101, 100).is_err());
    }
}
