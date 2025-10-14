//! Pure validation functions for game parameters.
//!
//! This module contains validation logic with no I/O dependencies,
//! used by both CLI and web interfaces.

/// Maximum allowed value for min/max range
pub const MAX_RANGE: i32 = 1_000_000;

/// Maximum guess limit for web/API (lower for security)
pub const MAX_WEB_GUESS_LIMIT: u32 = 100;

/// Maximum guess limit for CLI
pub const MAX_CLI_GUESS_LIMIT: u32 = 1000;

/// Validates a minimum value
pub fn validate_min_value(min: i32) -> Result<(), String> {
    if min < 0 {
        return Err(format!(
            "Minimum value ({}) must be non-negative (>= 0)",
            min
        ));
    }
    if min > MAX_RANGE {
        return Err(format!(
            "Minimum value ({}) exceeds maximum allowed ({})",
            min, MAX_RANGE
        ));
    }
    Ok(())
}

/// Validates a maximum value
pub fn validate_max_value(max: i32) -> Result<(), String> {
    if max < 0 {
        return Err(format!(
            "Maximum value ({}) must be non-negative (>= 0)",
            max
        ));
    }
    if max > MAX_RANGE {
        return Err(format!(
            "Maximum value ({}) exceeds maximum allowed ({})",
            max, MAX_RANGE
        ));
    }
    Ok(())
}

/// Validates that max >= min
pub fn validate_max_gte_min(min: i32, max: i32) -> Result<(), String> {
    if max < min {
        return Err(format!(
            "Maximum ({}) must be greater than or equal to minimum ({})",
            max, min
        ));
    }
    Ok(())
}

/// Validates both min and max values together
pub fn validate_range(min: i32, max: i32) -> Result<(), String> {
    validate_min_value(min)?;
    validate_max_value(max)?;
    validate_max_gte_min(min, max)?;
    Ok(())
}

/// Validates a guess limit and returns the adjusted limit (or None for no limit)
pub fn validate_guess_limit(limit: u32, max_limit: u32) -> Result<Option<u32>, String> {
    if limit == 0 {
        return Ok(None); // 0 means no limit
    }
    if limit > max_limit {
        return Err(format!(
            "Guess limit ({}) exceeds maximum allowed ({})",
            limit, max_limit
        ));
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
        assert!(validate_min_value(-1).is_err());
        assert!(validate_min_value(MAX_RANGE + 1).is_err());
    }

    #[test]
    fn test_validate_max_value() {
        assert!(validate_max_value(0).is_ok());
        assert!(validate_max_value(100).is_ok());
        assert!(validate_max_value(MAX_RANGE).is_ok());
        assert!(validate_max_value(-1).is_err());
        assert!(validate_max_value(MAX_RANGE + 1).is_err());
    }

    #[test]
    fn test_validate_max_gte_min() {
        assert!(validate_max_gte_min(0, 0).is_ok());
        assert!(validate_max_gte_min(0, 100).is_ok());
        assert!(validate_max_gte_min(100, 0).is_err());
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
