//! Pure calculation logic for difficulty metrics.
//!
//! This module contains the core algorithm for calculating optimal guesses
//! using binary search strategy, with no dependencies on I/O or web frameworks.

/// Calculates the optimal number of guesses needed using binary search strategy.
///
/// This function determines how many guesses a perfect binary search strategy
/// would require to guarantee finding a number in the given range.
///
/// # Algorithm
/// The optimal number of guesses is ceil(log2(range_size)).
/// We calculate this iteratively without floating point arithmetic to ensure precision.
///
/// # Examples
/// ```
/// use number_guessing_game::features::difficulty::calculate_optimal_guesses;
///
/// assert_eq!(calculate_optimal_guesses(1, 10), 4);   // 10 numbers -> 4 guesses
/// assert_eq!(calculate_optimal_guesses(1, 100), 7);  // 100 numbers -> 7 guesses
/// assert_eq!(calculate_optimal_guesses(1, 1000), 10); // 1000 numbers -> 10 guesses
/// ```
pub fn calculate_optimal_guesses(min: i32, max: i32) -> u32 {
    // Widen to i64: `max - min + 1` overflows i32 for extreme inputs
    // (e.g. min=0, max=i32::MAX). Callers validate against MAX_RANGE, but
    // this function must not panic on unvalidated input.
    let range_size = i64::from(max) - i64::from(min) + 1;

    // Edge case: if range has 1 or fewer numbers, only need 1 guess
    if range_size <= 1 {
        return 1;
    }

    // Calculate ceil(log2(range_size)) iteratively
    // Each guess cuts the search space in half (rounding up)
    let mut guesses = 0;
    let mut remaining = range_size as u64;

    while remaining > 1 {
        remaining = remaining.div_ceil(2);
        guesses += 1;
    }

    guesses
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_optimal_guesses_standard_ranges() {
        // Standard ranges from the plan
        assert_eq!(calculate_optimal_guesses(1, 10), 4);
        assert_eq!(calculate_optimal_guesses(1, 100), 7);
        assert_eq!(calculate_optimal_guesses(1, 1000), 10);
        assert_eq!(calculate_optimal_guesses(1, 10000), 14);
    }

    #[test]
    fn test_calculate_optimal_guesses_single_number() {
        // When min equals max, only 1 guess needed
        assert_eq!(calculate_optimal_guesses(5, 5), 1);
        assert_eq!(calculate_optimal_guesses(0, 0), 1);
        assert_eq!(calculate_optimal_guesses(100, 100), 1);
    }

    #[test]
    fn test_calculate_optimal_guesses_small_ranges() {
        assert_eq!(calculate_optimal_guesses(1, 2), 1); // 2 numbers -> 1 guess
        assert_eq!(calculate_optimal_guesses(1, 3), 2); // 3 numbers -> 2 guesses
        assert_eq!(calculate_optimal_guesses(1, 4), 2); // 4 numbers -> 2 guesses
        assert_eq!(calculate_optimal_guesses(1, 5), 3); // 5 numbers -> 3 guesses
        assert_eq!(calculate_optimal_guesses(1, 8), 3); // 8 numbers -> 3 guesses
    }

    #[test]
    fn test_calculate_optimal_guesses_power_of_two() {
        // Powers of 2 are exact (no ceiling needed)
        assert_eq!(calculate_optimal_guesses(1, 2), 1); // 2^1
        assert_eq!(calculate_optimal_guesses(1, 4), 2); // 2^2
        assert_eq!(calculate_optimal_guesses(1, 8), 3); // 2^3
        assert_eq!(calculate_optimal_guesses(1, 16), 4); // 2^4
        assert_eq!(calculate_optimal_guesses(1, 32), 5); // 2^5
        assert_eq!(calculate_optimal_guesses(1, 64), 6); // 2^6
        assert_eq!(calculate_optimal_guesses(1, 128), 7); // 2^7
    }

    #[test]
    fn test_calculate_optimal_guesses_arbitrary_min() {
        // Range size is what matters, not the starting point
        assert_eq!(calculate_optimal_guesses(10, 19), 4); // 10 numbers
        assert_eq!(calculate_optimal_guesses(50, 149), 7); // 100 numbers
        assert_eq!(calculate_optimal_guesses(500, 1499), 10); // 1000 numbers
    }

    #[test]
    fn test_calculate_optimal_guesses_very_large_range() {
        // Test the upper limit (1,000,000)
        assert_eq!(calculate_optimal_guesses(0, 999999), 20); // 1,000,000 numbers
        assert_eq!(calculate_optimal_guesses(0, 1000000), 20); // 1,000,001 numbers
    }

    #[test]
    fn test_calculate_optimal_guesses_extreme_inputs_no_overflow() {
        // Regression: `max - min + 1` previously overflowed i32 and panicked
        // in debug builds for unvalidated extreme inputs.
        assert_eq!(calculate_optimal_guesses(0, i32::MAX), 31); // 2^31 numbers
        assert_eq!(calculate_optimal_guesses(i32::MIN, i32::MAX), 32); // 2^32 numbers
    }
}
