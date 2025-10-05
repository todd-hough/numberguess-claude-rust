# Game State Boundary Conditions

This document provides a comprehensive analysis of all boundary conditions in the number guessing game, including value limits, state transitions, and temporal changes.

## Table of Contents
- [Numeric Range Boundaries](#numeric-range-boundaries)
- [Game State Boundaries](#game-state-boundaries)
- [Guess Limit Boundaries](#guess-limit-boundaries)
- [Database State Boundaries](#database-state-boundaries)
- [State Transition Boundaries](#state-transition-boundaries)
- [Concurrency Boundaries](#concurrency-boundaries)

---

## Numeric Range Boundaries

| Parameter | Minimum Value | Maximum Value | Invalid Values | Notes |
|-----------|--------------|---------------|----------------|-------|
| `min` | 0 | 1,000,000 | < 0, > 1,000,000 | Must be non-negative |
| `max` | 0 | 1,000,000 | < 0, > 1,000,000 | Must be non-negative |
| `max >= min` | min value | 1,000,000 | max < min | Range validation |
| `secret_number` | min | max | < min, > max | Generated or validated within range |
| `guess` | -∞ | +∞ | None (i32 valid) | No explicit validation in game logic |

### Edge Cases
- **Zero range**: `min = max = 0` is **valid** (secret must be 0)
- **Single value range**: `min = max = N` is **valid** (secret must be N)
- **Maximum range**: `min = 0, max = 1,000,000` is **valid**
- **Overflow protection**: Range too large check prevents subtraction overflow

---

## Game State Boundaries

| State Field | Type | Minimum | Maximum | Edge Cases |
|-------------|------|---------|---------|------------|
| `guess_count` | u32 | 0 | u32::MAX (4,294,967,295) | Increments with each guess |
| `max_guesses` | Option\<u32\> | None or 1 | CLI: 1000, Web: 100 | None = unlimited |
| `game_id` | u64 | 0 | u64::MAX | Random generation, collisions unlikely |

### State Invariants
1. `guess_count <= max_guesses.unwrap_or(u32::MAX)` must hold during game
2. `secret_number` must be in `[min, max]` (validated in `from_db()`)
3. Active games have `guess_count < max_guesses` OR `max_guesses == None`

---

## Guess Limit Boundaries

| Context | Minimum Limit | Maximum Limit | Special Values | Behavior |
|---------|--------------|---------------|----------------|----------|
| CLI | 0 (no limit) | 1,000 | 0 → None | Enforced by validator |
| Web/API | 0 (no limit) | 100 | 0 → None | Lower for security |
| Internal | None | u32::MAX | None → Unlimited | Game logic |

### Limit State Transitions

| Guesses Made | Limit | has_guesses_remaining() | Next Guess Result |
|-------------|-------|------------------------|-------------------|
| 0 | Some(3) | true | TooLow/TooHigh/Correct |
| 1 | Some(3) | true | TooLow/TooHigh/Correct |
| 2 | Some(3) | true | TooLow/TooHigh/Correct/LimitReached* |
| 3 | Some(3) | false | LimitReached (immediately) |
| 3 | None | true | TooLow/TooHigh/Correct |
| 100 | None | true | TooLow/TooHigh/Correct |

\* LimitReached only if guess is incorrect on the final allowed attempt

---

## Database State Boundaries

### Field Constraints

| Database Column | SQL Type | Rust Type | Min Value | Max Value | Notes |
|----------------|----------|-----------|-----------|-----------|-------|
| `game_id` | BIGINT | i64 | -9,223,372,036,854,775,808 | 9,223,372,036,854,775,807 | GameId internally uses u64 |
| `min_value` | INTEGER | i32 | -2,147,483,648 | 2,147,483,647 | Application constrains to [0, 1,000,000] |
| `max_value` | INTEGER | i32 | -2,147,483,648 | 2,147,483,647 | Application constrains to [0, 1,000,000] |
| `secret_number` | INTEGER | i32 | -2,147,483,648 | 2,147,483,647 | Constrained to [min_value, max_value] |
| `guess_count` | INTEGER | i32 → u32 | 0 | 2,147,483,647 | Negative values caught in conversion |
| `max_guesses` | INTEGER NULL | i32 → u32 | 0 or NULL | 100 (web) / 1000 (CLI) | NULL = unlimited |
| `created_at` | TIMESTAMP | - | 1970-01-01 | 2038-01-19 (32-bit) | Auto-set |
| `updated_at` | TIMESTAMP | - | 1970-01-01 | 2038-01-19 (32-bit) | Auto-updated |

### Database Type Conversion Edge Cases

| Scenario | Database Value | Conversion | Result |
|----------|---------------|------------|---------|
| Negative guess_count | -1 (i32) | → u32 | DbError::ConversionError |
| Negative max_guesses | -1 (i32) | → u32 | DbError::ConversionError |
| NULL max_guesses | NULL | → Option\<u32\> | Ok(None) |
| Max i32 guess_count | 2,147,483,647 | → u32 | Ok(2,147,483,647) |

---

## State Transition Boundaries

### Game Lifecycle States

```
[NOT EXIST] → [ACTIVE] → [COMPLETED/DELETED]
     ↓            ↓              ↓
  (create)    (guessing)    (game over)
```

| From State | To State | Trigger | Database Action | Game Object |
|-----------|----------|---------|-----------------|-------------|
| Non-existent | Active | create_game() | INSERT | Created with guess_count=0 |
| Active (guesses < limit) | Active | make_guess() → TooLow/TooHigh | UPDATE guess_count | guess_count++ |
| Active (last guess) | Completed | make_guess() → Correct | DELETE | Object discarded |
| Active (last guess) | Completed | make_guess() → LimitReached | DELETE | Object discarded |
| Active (limit reached) | Completed | make_guess() | No DB access | LimitReached immediately |
| Completed | N/A | get_game() | No record | DbError::NotFound |
| Completed | N/A | make_guess() | No record | DbError::NotFound |

### Critical Boundary: Final Guess

**When `guess_count = max_guesses - 1` (one guess remaining):**

| Guess Result | guess_count After | has_guesses_remaining() | Return Value | DB Action |
|-------------|-------------------|------------------------|--------------|-----------|
| Correct | N/A | N/A | GuessResult::Correct | DELETE |
| TooLow/TooHigh | max_guesses | false | GuessResult::LimitReached | DELETE |

**Logic flow in `make_guess()`:**
1. Check `has_guesses_remaining()` before guess → If false, return LimitReached immediately
2. Increment `guess_count`
3. Compare guess to secret
4. If incorrect AND now `!has_guesses_remaining()`, return LimitReached
5. Otherwise return comparison result

---

## Concurrency Boundaries

### Race Conditions (Mitigated)

| Scenario | Without Transaction | With Transaction (FOR UPDATE) |
|----------|-------------------|------------------------------|
| Two guesses at same time | Both read guess_count=5, both write 6 | Second blocks until first commits |
| Guess after game ends | May guess on deleted game | Returns NotFound (row locked then deleted) |
| Simultaneous final guesses | Both could see same guess_count | Only one processes, other waits |

### Transaction Lock Boundaries

| Action | Lock Type | Duration | Blocks |
|--------|-----------|----------|--------|
| `make_guess_transactional()` | Row-level (FOR UPDATE) | Transaction duration | Other guesses on same game_id |
| `get_game()` | Shared read | Query duration | Nothing |
| `create_game()` | None (new row) | Insert only | Nothing |

---

## Temporal Boundaries

### Game Existence Over Time

| Time Point | State | Database | Actions Allowed |
|-----------|-------|----------|-----------------|
| Before creation | Non-existent | No row | create_game() only |
| Just created | Active, guess_count=0 | Row exists | get_game(), make_guess() |
| During gameplay | Active, 0 < guess_count < max_guesses | Row exists, updated_at changes | get_game(), make_guess() |
| Final guess (correct) | Transitioning | Row being deleted | Concurrent guesses blocked (locked) |
| Final guess (limit) | Transitioning | Row being deleted | Concurrent guesses blocked (locked) |
| After completion | Non-existent | No row | get_game() → NotFound, make_guess() → NotFound |
| Server restart | Persistent (if DB used) | Rows preserved | All active games restored |

### Timestamp Boundaries

| Field | Set On | Updated On | Precision |
|-------|--------|------------|-----------|
| `created_at` | INSERT | Never | Microsecond (PostgreSQL) |
| `updated_at` | INSERT (NOW()) | Every guess (UPDATE) | Microsecond (PostgreSQL) |

---

## Error Boundaries

### Game Creation Errors

| Condition | Error Type | Error Message |
|-----------|-----------|---------------|
| min < 0 | GameError::NegativeMin | "Minimum value ({}) must be non-negative (>= 0)" |
| max < 0 | GameError::NegativeMax | "Maximum value ({}) must be non-negative (>= 0)" |
| max < min | GameError::InvalidRange | "Maximum ({}) must be greater than or equal to minimum ({})" |
| min > 1,000,000 | GameError::MinExceedsLimit | "Minimum value ({}) exceeds maximum allowed value (1000000)" |
| max > 1,000,000 | GameError::MaxExceedsLimit | "Maximum value ({}) exceeds maximum allowed value (1000000)" |
| max - min = i32::MAX | GameError::RangeTooLarge | "Range between min ({}) and max ({}) is too large" |

### Database Reconstruction Errors

| Condition | Error Type | Error Message |
|-----------|-----------|---------------|
| secret < min or secret > max | GameError::SecretOutOfRange | "Secret number ({}) must be between min ({}) and max ({})" |
| guess_count < 0 (i32) | DbError::ConversionError | "Guess count is negative" |
| max_guesses < 0 (i32) | DbError::ConversionError | "Max guesses is negative" |
| Row not found | DbError::NotFound | "Game not found" |

### Runtime State Errors

| Condition | When | Behavior |
|-----------|------|----------|
| make_guess() on completed game | After game deleted from DB | Returns DbError::NotFound |
| make_guess() when limit reached | guess_count >= max_guesses | Returns GuessResult::LimitReached immediately |
| Concurrent guess during deletion | Transaction in progress | Blocks until commit, then NotFound |

---

## Summary Table: Critical Boundaries

| Category | Boundary Condition | Expected Behavior | Tested? |
|----------|-------------------|-------------------|---------|
| Range | min = max = 0 | Valid game, secret = 0 | ✓ (test_zero_values_allowed) |
| Range | min = 0, max = 1,000,000 | Valid maximum range | ✓ (test_large_valid_range) |
| Range | max < min | GameError::InvalidRange | ✓ (test_invalid_range) |
| Guesses | guess_count = max_guesses - 1, wrong guess | LimitReached | ✓ (test_game_with_guess_limit) |
| Guesses | guess_count = max_guesses - 1, correct guess | Correct with attempts | ✓ (test_correct_guess_within_limit) |
| Guesses | max_guesses = None | Unlimited guesses | ✓ (test_game_with_no_limit) |
| Guesses | guess after limit reached | LimitReached immediately | ✓ (test_game_with_guess_limit) |
| Database | Game not in DB | DbError::NotFound | ✓ (integration tests) |
| Database | Completed game (deleted) | DbError::NotFound | ✓ (game_lifecycle_test) |
| Database | Negative guess_count from DB | DbError::ConversionError | Implicit (type safety) |
| Concurrency | Simultaneous guesses | Serialized by transaction lock | ✓ (concurrent_games_test) |
| Validation | CLI limit > 1000 | Validation error | ✓ (validators tests) |
| Validation | Web limit > 100 | Validation error | ✓ (validators tests) |

---

## Testing Recommendations

### Additional Boundary Tests Needed

1. **Database timestamp boundaries**: Test games created near year 2038 (if using 32-bit timestamps)
2. **Game ID collision**: Test behavior when random game_id collides (extremely rare)
3. **Maximum guess_count**: Test game with u32::MAX guesses (theoretical)
4. **Exactly at limit**: Test guess_count = max_guesses without exceeding
5. **Type conversion edges**: Test i32::MAX values in database fields

### Coverage Gaps

| Boundary | Current Test | Gap |
|----------|-------------|-----|
| Zero guess limit in web | Validator test exists | No integration test for 0 → None conversion |
| Database cleanup on limit reached | Integration test exists | Could add explicit verification |
| Concurrent create + guess | Not explicitly tested | May want to add |
| Updated_at precision | Not tested | Could verify timestamp updates |

---

## Maintenance Notes

When modifying the game:

1. **Changing limits**: Update `MAX_RANGE`, `MAX_WEB_GUESS_LIMIT`, `MAX_CLI_GUESS_LIMIT` in `validators.rs`
2. **Adding fields**: Update both `GuessingGame` struct and database schema + migrations
3. **Changing state transitions**: Review `make_guess()` logic and `make_guess_transactional()`
4. **New error conditions**: Add to `GameError` or `DbError` enums with proper Display messages

## Version Information

- **Document Version**: 1.0
- **Last Updated**: 2025-10-05
- **Rust Version**: 1.89.0
- **Database**: PostgreSQL with SQLx runtime checking
