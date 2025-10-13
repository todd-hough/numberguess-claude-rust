# Interactive Difficulty Indicator Feature

**Created**: 2025-10-12
**Updated**: 2025-10-12
**Status**: Ready for Implementation
**Priority**: High
**Complexity**: Low
**Implementation Time**: 1-2 days
**Implementation Approach**: HTMX + Rust (server-side)

---

## Overview

Add real-time difficulty feedback to the game creation page that dynamically updates as users adjust the min/max range and guess limit. This provides educational feedback about game difficulty and helps users make informed choices.

### Key Concept

Instead of preset difficulty buttons, provide **live feedback** that shows users how difficult their custom game will be. As they type in different values, the difficulty indicator updates in real-time via HTMX, teaching them about binary search efficiency.

### Technical Approach

**HTMX + Rust (Server-Side Calculation)**
- All logic stays in Rust (no JavaScript/Rust split)
- HTMX triggers on input change with debouncing
- Server calculates difficulty and returns HTML fragment
- Consistent with existing architecture (HTMX for dynamic updates)
- Type-safe, testable, maintainable

---

## User Experience Goals

1. **Educational** - Teach players about optimal binary search strategy
2. **Helpful** - Guide users to create appropriately challenging games
3. **Non-intrusive** - Enhance, don't replace, the custom input experience
4. **Responsive** - Update smoothly as inputs change (300ms debounce)
5. **Accessible** - Clear visual and textual feedback

---

## Architecture & Separation of Concerns

This feature follows the same clean architecture as the rest of the codebase:

### Layer 1: Pure Logic (`src/difficulty.rs`)
**Purpose**: Calculate difficulty metrics (no I/O, no web dependencies)

Similar to `src/game.rs` and `src/validators.rs` - pure functions and types.

**Modules**:
```rust
pub enum DifficultyLevel {
    Unlimited, VeryEasy, Easy, Medium, Hard, Expert, Impossible
}

pub struct DifficultyInfo {
    pub min: i32,
    pub max: i32,
    pub range_size: u32,
    pub optimal_guesses: u32,
    pub guess_limit: Option<u32>,
    pub buffer: i32,
    pub level: DifficultyLevel,
}

pub fn calculate_optimal_guesses(min: i32, max: i32) -> u32;
pub fn calculate_difficulty(min: i32, max: i32, limit: Option<u32>) -> DifficultyInfo;
```

### Layer 2: Template Structs (`src/templates.rs`)
**Purpose**: Type-safe data for Askama rendering

```rust
#[derive(Template)]
#[template(path = "difficulty_indicator.html")]
pub struct DifficultyIndicator {
    pub info: DifficultyInfo,
}
```

### Layer 3: Askama Template (`templates/difficulty_indicator.html`)
**Purpose**: HTML rendering with type safety

Receives `DifficultyInfo` and renders the visual display.

### Layer 4: Web Endpoint (`src/web.rs`)
**Purpose**: Thin HTTP handler (no business logic)

```rust
async fn difficulty_preview(
    Query(params): Query<DifficultyParams>,
) -> impl IntoResponse {
    // Validate inputs
    // Call difficulty::calculate_difficulty()
    // Render template
    // Return HTML fragment
}
```

### Layer 5: HTMX Integration (`static/index.html`)
**Purpose**: Client-side triggers

```html
<input name="min"
       hx-get="/difficulty-preview"
       hx-trigger="input changed delay:300ms, load"
       hx-include="[name='min'],[name='max'],[name='max_guesses']"
       hx-target="#difficulty-preview">
```

**Benefits of This Architecture**:
- ✅ All logic in Rust (no JavaScript duplication)
- ✅ Unit testable (difficulty calculations)
- ✅ Integration testable (HTTP endpoint)
- ✅ Type-safe throughout
- ✅ Server-side validation
- ✅ Consistent with existing codebase
- ✅ No client-side state management

---

## Core Calculations

### Optimal Guesses (Binary Search)

The optimal number of guesses using binary search strategy:

```rust
pub fn calculate_optimal_guesses(min: i32, max: i32) -> u32 {
    let range_size = (max - min + 1) as u32;

    if range_size <= 1 {
        return 1;
    }

    // Calculate ceil(log2(range_size))
    let mut guesses = 0;
    let mut remaining = range_size;

    while remaining > 1 {
        remaining = (remaining + 1) / 2; // Ceiling division
        guesses += 1;
    }

    guesses
}
```

**Examples**:
- Range 1-10 (10 numbers) → 4 guesses optimal
- Range 1-100 (100 numbers) → 7 guesses optimal
- Range 1-1000 (1000 numbers) → 10 guesses optimal
- Range 1-10000 (10000 numbers) → 14 guesses optimal

### Difficulty Levels

Based on the "buffer" between guess limit and optimal:

| Buffer | Difficulty Level | Description |
|--------|-----------------|-------------|
| No limit | **Unlimited** | Can experiment freely |
| ≥ 5 | **Very Easy** | Great for beginners |
| 3-4 | **Easy** | Comfortable challenge |
| 2 | **Medium** | Balanced difficulty |
| 1 | **Hard** | Challenging |
| 0 | **Expert** | Perfect play required |
| < 0 | **Impossible** | Below optimal (nearly impossible) |

---

## UI Design

### Selected Layout: Sidebar Design (Option 2)

```
┌──────────────────┬──────────────────────────────┐
│ Setup            │  Difficulty Preview          │
│                  │                              │
│ Min: [1    ]     │  ██████████ MEDIUM 🎯        │
│ Max: [100  ]     │                              │
│ Limit: [10 ]     │  Range: 100 numbers          │
│                  │  Optimal: 7 guesses          │
│ [Start Game]     │  Your limit: 10              │
│                  │  Buffer: 3 extra             │
│                  │                              │
│                  │  ✓ Good balance of           │
│                  │    challenge and fun!        │
└──────────────────┴──────────────────────────────┘
```

**Why Sidebar?**
- Clean separation of form and preview
- Doesn't interrupt form flow
- Easy to glance at while adjusting inputs
- Works well on desktop (stacks on mobile)

---

## Visual Elements

### 1. Difficulty Badge with Meter

```html
<div class="difficulty-header">
    <span class="difficulty-badge {{ info.level.css_class() }}"
          style="background: {{ info.level.color() }};">
        <span class="difficulty-icon">{{ info.level.icon() }}</span>
        <span class="difficulty-name">{{ info.level.name() }}</span>
    </span>
    <div class="difficulty-meter-container">
        <div class="difficulty-meter"
             style="width: {{ info.level.meter_width() }}%; background: {{ info.level.color() }};"></div>
    </div>
</div>
```

### 2. Stats Display

```html
<div class="difficulty-stats">
    <div class="stat-row">
        <span class="stat-label">📊 Range:</span>
        <span class="stat-value">{{ info.range_size_description() }}</span>
    </div>
    <div class="stat-row">
        <span class="stat-label">🎯 Optimal:</span>
        <span class="stat-value">{{ info.optimal_description() }}</span>
    </div>
    {% if info.has_limit() %}
    <div class="stat-row">
        <span class="stat-label">🎲 Your limit:</span>
        <span class="stat-value">{{ info.limit_description() }}</span>
    </div>
    <div class="stat-row">
        <span class="stat-label">💡 Buffer:</span>
        <span class="stat-value">{{ info.buffer_description() }}</span>
    </div>
    {% endif %}
</div>
```

### 3. Contextual Messages

Difficulty-specific messages rendered from `info.level.message()`:

| Difficulty | Message |
|------------|---------|
| **Unlimited** | "Take your time! No guess limit means you can experiment freely." |
| **Very Easy** | "Great for beginners! You have plenty of room to learn." |
| **Easy** | "A comfortable challenge with room for mistakes." |
| **Medium** | "Good balance of challenge and fun! Use a smart strategy." |
| **Hard** | "This is challenging! You'll need an efficient approach to win." |
| **Expert** | "No room for error! Perfect binary search required." |
| **Impossible** | "Your limit is below optimal. You'd need perfect play plus incredible luck!" |

---

## Implementation Details

### Step 1: Create Difficulty Module (`src/difficulty.rs`)

New file with pure calculation logic.

**Key Types**:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum DifficultyLevel {
    Unlimited,
    VeryEasy,
    Easy,
    Medium,
    Hard,
    Expert,
    Impossible,
}

impl DifficultyLevel {
    pub fn name(&self) -> &'static str { /* ... */ }
    pub fn icon(&self) -> &'static str { /* ... */ }
    pub fn color(&self) -> &'static str { /* ... */ }
    pub fn meter_width(&self) -> u8 { /* ... */ }
    pub fn message(&self) -> &'static str { /* ... */ }
    pub fn css_class(&self) -> &'static str { /* ... */ }
}

#[derive(Debug, Clone)]
pub struct DifficultyInfo {
    pub min: i32,
    pub max: i32,
    pub range_size: u32,
    pub optimal_guesses: u32,
    pub guess_limit: Option<u32>,
    pub buffer: i32,
    pub level: DifficultyLevel,
}

impl DifficultyInfo {
    pub fn has_limit(&self) -> bool { /* ... */ }
    pub fn buffer_description(&self) -> String { /* ... */ }
    pub fn range_size_description(&self) -> String { /* ... */ }
    pub fn optimal_description(&self) -> String { /* ... */ }
    pub fn limit_description(&self) -> String { /* ... */ }
}
```

**Key Functions**:
```rust
pub fn calculate_optimal_guesses(min: i32, max: i32) -> u32 {
    // Binary search calculation (iterative, no floating point)
}

pub fn calculate_difficulty(min: i32, max: i32, guess_limit: Option<u32>) -> DifficultyInfo {
    // Calculate optimal, buffer, determine level
}
```

### Step 2: Update `src/lib.rs`

Export the new module:

```rust
pub mod difficulty;
```

### Step 3: Add Template Struct (`src/templates.rs`)

```rust
use crate::difficulty::DifficultyInfo;

#[derive(Template)]
#[template(path = "difficulty_indicator.html")]
pub struct DifficultyIndicator {
    pub info: DifficultyInfo,
}
```

### Step 4: Create Askama Template (`templates/difficulty_indicator.html`)

```html
<div class="difficulty-box">
    <div class="difficulty-header">
        <span class="difficulty-badge {{ info.level.css_class() }}"
              style="background: {{ info.level.color() }};">
            <span class="difficulty-icon">{{ info.level.icon() }}</span>
            <span class="difficulty-name">{{ info.level.name() }}</span>
        </span>
        <div class="difficulty-meter-container">
            <div class="difficulty-meter"
                 style="width: {{ info.level.meter_width() }}%; background: {{ info.level.color() }};"></div>
        </div>
    </div>

    <div class="difficulty-stats">
        <div class="stat-row">
            <span class="stat-label">📊 Range:</span>
            <span class="stat-value">{{ info.range_size_description() }}</span>
        </div>
        <div class="stat-row">
            <span class="stat-label">🎯 Optimal:</span>
            <span class="stat-value">{{ info.optimal_description() }}</span>
        </div>
        {% if info.has_limit() %}
        <div class="stat-row">
            <span class="stat-label">🎲 Your limit:</span>
            <span class="stat-value">{{ info.limit_description() }}</span>
        </div>
        <div class="stat-row">
            <span class="stat-label">💡 Buffer:</span>
            <span class="stat-value">{{ info.buffer_description() }}</span>
        </div>
        {% endif %}
    </div>

    <div class="difficulty-message">
        {{ info.level.message() }}
    </div>
</div>
```

### Step 5: Add Web Endpoint (`src/web.rs`)

```rust
use crate::difficulty::{calculate_difficulty, DifficultyInfo};
use crate::templates::DifficultyIndicator;

#[derive(Debug, Deserialize)]
struct DifficultyParams {
    min: Option<i32>,
    max: Option<i32>,
    max_guesses: Option<u32>,
}

async fn difficulty_preview(
    Query(params): Query<DifficultyParams>,
) -> impl IntoResponse {
    // Extract and validate inputs
    let min = params.min.unwrap_or(1);
    let max = params.max.unwrap_or(100);

    // Validate range
    if min < 0 || max < 0 || max < min {
        // Return empty response for invalid inputs
        return Html("".to_string());
    }

    // Calculate difficulty
    let info = calculate_difficulty(min, max, params.max_guesses);

    // Render template
    let template = DifficultyIndicator { info };
    Html(template.render().unwrap_or_default())
}

// Add route to router
let app = Router::new()
    // ... existing routes ...
    .route("/difficulty-preview", get(difficulty_preview))
    // ...
```

### Step 6: Update `static/index.html`

**Add HTMX attributes to inputs**:

```html
<div class="form-container">
    <form class="game-form" hx-post="/game/new" hx-target=".container" hx-swap="innerHTML">
        <div class="form-group">
            <label for="min">Minimum Number (0 to 1,000,000):</label>
            <input type="number" id="min" name="min" value="1"
                   min="0" max="1000000" required
                   hx-get="/difficulty-preview"
                   hx-trigger="input changed delay:300ms, load"
                   hx-include="[name='min'],[name='max'],[name='max_guesses']"
                   hx-target="#difficulty-preview">
        </div>
        <div class="form-group">
            <label for="max">Maximum Number (0 to 1,000,000):</label>
            <input type="number" id="max" name="max" value="100"
                   min="0" max="1000000" required
                   hx-get="/difficulty-preview"
                   hx-trigger="input changed delay:300ms, load"
                   hx-include="[name='min'],[name='max'],[name='max_guesses']"
                   hx-target="#difficulty-preview">
        </div>
        <div class="form-group">
            <label for="max_guesses">Guess Limit (Optional, max 100):</label>
            <input type="number" id="max_guesses" name="max_guesses"
                   min="0" max="100" placeholder="Leave blank for no limit"
                   hx-get="/difficulty-preview"
                   hx-trigger="input changed delay:300ms, load"
                   hx-include="[name='min'],[name='max'],[name='max_guesses']"
                   hx-target="#difficulty-preview">
        </div>
        <button type="submit">Start New Game</button>
    </form>

    <!-- Difficulty preview sidebar -->
    <div id="difficulty-preview" class="difficulty-preview">
        <!-- HTMX will populate this on load and input change -->
    </div>
</div>
```

**Add CSS for sidebar layout**:

```css
.form-container {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 30px;
}

@media (max-width: 768px) {
    .form-container {
        grid-template-columns: 1fr;
    }
}

.difficulty-preview {
    padding: 20px;
    background: #f8f9fa;
    border-radius: 12px;
    align-self: start;
    position: sticky;
    top: 20px;
}

.difficulty-box {
    /* Styles from previous section */
}

/* ... rest of CSS ... */
```

---

## Why HTMX Instead of JavaScript?

### Benefits

1. **No Code Duplication**
   - All logic in Rust (one source of truth)
   - No need to maintain parallel JS calculations

2. **Type Safety**
   - Compile-time validation of calculations
   - Type-safe template rendering
   - No runtime type errors

3. **Testability**
   - Unit test `difficulty::calculate_optimal_guesses()`
   - Unit test `difficulty::calculate_difficulty()`
   - Integration test `/difficulty-preview` endpoint
   - Standard HTTP testing tools

4. **Consistency**
   - Same architecture as rest of app (HTMX for dynamic updates)
   - No context switching between Rust and JS
   - Easier for Rust devs to maintain

5. **Server-Side Validation**
   - Input validation happens server-side
   - No client-side validation bypass
   - Consistent with game creation validation

6. **Simplicity**
   - No build step for JS
   - No bundling needed
   - Just HTML + HTMX attributes

### Trade-offs

- **Slightly more server load**: Each input change makes HTTP request
  - Mitigated by 300ms debounce
  - Calculations are O(1) and trivial
  - Expected load: negligible

- **Network latency**: Small delay vs instant JS
  - Mitigated by debouncing (feels instant)
  - Typical latency: < 10ms for local server
  - Acceptable for preview feature

**Verdict**: Benefits far outweigh trade-offs for this application.

---

## Testing Strategy

### Unit Tests (`src/difficulty.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_optimal_guesses() {
        assert_eq!(calculate_optimal_guesses(1, 10), 4);
        assert_eq!(calculate_optimal_guesses(1, 100), 7);
        assert_eq!(calculate_optimal_guesses(1, 1000), 10);
        assert_eq!(calculate_optimal_guesses(5, 5), 1);  // Min = max
    }

    #[test]
    fn test_difficulty_levels() {
        let info = calculate_difficulty(1, 100, Some(12));
        assert_eq!(info.level, DifficultyLevel::VeryEasy);
        assert_eq!(info.buffer, 5);

        let info = calculate_difficulty(1, 100, Some(10));
        assert_eq!(info.level, DifficultyLevel::Easy);

        let info = calculate_difficulty(1, 100, Some(9));
        assert_eq!(info.level, DifficultyLevel::Medium);

        let info = calculate_difficulty(1, 100, Some(6));
        assert_eq!(info.level, DifficultyLevel::Impossible);
    }

    #[test]
    fn test_unlimited_difficulty() {
        let info = calculate_difficulty(1, 100, None);
        assert_eq!(info.level, DifficultyLevel::Unlimited);
        assert!(!info.has_limit());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_difficulty_preview_endpoint() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/difficulty-preview?min=1&max=100&max_guesses=10")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("Medium"));
    assert!(html.contains("100 numbers"));
    assert!(html.contains("7 guesses"));
}

#[tokio::test]
async fn test_difficulty_preview_invalid_inputs() {
    let app = create_test_app();

    // Max < Min should return empty
    let response = app
        .oneshot(
            Request::builder()
                .uri("/difficulty-preview?min=100&max=10")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(body.len(), 0); // Empty response
}
```

### Manual Testing Checklist

- [ ] Indicator appears on page load with default values
- [ ] Indicator updates after 300ms of typing pause
- [ ] Correct optimal calculation for various ranges:
  - [ ] 1-10 → 4 guesses
  - [ ] 1-100 → 7 guesses
  - [ ] 1-1000 → 10 guesses
- [ ] Difficulty levels display correctly:
  - [ ] Unlimited (no limit set)
  - [ ] Very Easy (buffer ≥ 5)
  - [ ] Easy (buffer ≥ 3)
  - [ ] Medium (buffer ≥ 2)
  - [ ] Hard (buffer = 1)
  - [ ] Expert (buffer = 0)
  - [ ] Impossible (buffer < 0)
- [ ] Colors change appropriately with difficulty
- [ ] Stats display correct numbers with proper pluralization
- [ ] Limit-specific stats hide when limit = 0
- [ ] Invalid inputs (min > max) show empty preview
- [ ] Works on mobile (sidebar stacks)
- [ ] Works with keyboard navigation

### Edge Cases

1. **Min = Max**: Range of 1, optimal = 1
2. **Very large range**: 0-1,000,000, optimal = 20
3. **Limit = 0**: Should show "Unlimited"
4. **Limit < Optimal**: Should show "Impossible"
5. **Empty inputs**: Should show empty preview
6. **Negative numbers**: Validation should prevent submission
7. **Non-numeric input**: Browser validation handles

---

## Accessibility

### ARIA Labels

```html
<div id="difficulty-preview"
     role="region"
     aria-live="polite"
     aria-label="Difficulty preview">

    <div class="difficulty-box">
        <div class="difficulty-header"
             role="status"
             aria-label="Difficulty: {{ info.level.name() }}">
            <!-- ... -->
        </div>

        <div class="difficulty-stats"
             aria-label="Game statistics">
            <!-- ... -->
        </div>
    </div>
</div>
```

### Screen Reader Experience

When difficulty updates:
> "Difficulty preview updated. Medium. Range: 100 numbers. Optimal: 7 guesses. Your limit: 10 guesses. 3 extra guesses."

### Keyboard Navigation

- Indicator updates as user tabs through inputs
- No interactive elements in preview (pure display)
- Doesn't trap focus
- All updates announced via `aria-live="polite"`

---

## Performance Considerations

- **Server Load**: Negligible
  - O(1) calculation (simple loop, no recursion)
  - < 1ms per request
  - Debounced to 300ms (max ~3 req/sec per user)

- **Network**: Minimal
  - Request size: ~100 bytes (query params)
  - Response size: ~1-2 KB (HTML fragment)
  - Gzipped: ~500 bytes

- **Client**: No JavaScript overhead
  - HTMX is already loaded
  - No additional JS execution
  - Simple DOM swap

---

## Implementation Timeline

### Day 1: Core Implementation (3-4 hours)
- [x] Create `src/difficulty.rs` with calculations
- [ ] Update `src/lib.rs` to export module
- [ ] Add `DifficultyIndicator` to `src/templates.rs`
- [ ] Create `templates/difficulty_indicator.html`
- [ ] Add `/difficulty-preview` endpoint to `src/web.rs`
- [ ] Unit tests for difficulty calculations

### Day 2: Integration & Polish (2-3 hours)
- [ ] Update `static/index.html` with HTMX triggers
- [ ] Add CSS for sidebar layout
- [ ] Integration tests for endpoint
- [ ] Manual testing (all difficulty levels)
- [ ] Edge case testing
- [ ] Accessibility testing
- [ ] Mobile responsive testing
- [ ] Update documentation

---

## Success Metrics

Track these to measure feature impact:

| Metric | Baseline | Target | Indicates |
|--------|----------|--------|-----------|
| Game creation rate | - | +10% | Less intimidation |
| Average game completion | - | +15% | Better balanced games |
| Games with limits set | - | +20% | More engagement with feature |
| Time spent on setup page | - | +30% | Users experimenting |
| Custom range variety | - | +25% | More exploration |
| Win rate | - | Stable | Games are well-balanced |

---

## Example Screenshots

### Medium Difficulty (1-100, limit 10)
```
┌──────────────────┬──────────────────────────────┐
│ Setup            │  Difficulty Preview          │
│                  │                              │
│ Min: [1    ]     │  ██████████ MEDIUM 🎯        │
│ Max: [100  ]     │                              │
│ Limit: [10 ]     │  Range: 100 numbers          │
│                  │  Optimal: 7 guesses          │
│ [Start Game]     │  Your limit: 10 guesses      │
│                  │  Buffer: 3 extra guesses     │
│                  │                              │
│                  │  Good balance of challenge   │
│                  │  and fun! Use a smart        │
│                  │  strategy.                   │
└──────────────────┴──────────────────────────────┘
```

### Expert Difficulty (1-1000, limit 10)
```
┌──────────────────┬──────────────────────────────┐
│ Setup            │  Difficulty Preview          │
│                  │                              │
│ Min: [1     ]    │  ███████████ EXPERT ⚡        │
│ Max: [1000  ]    │                              │
│ Limit: [10  ]    │  Range: 1,000 numbers        │
│                  │  Optimal: 10 guesses         │
│ [Start Game]     │  Your limit: 10 guesses      │
│                  │  Buffer: 0 extra guesses     │
│                  │                              │
│                  │  No room for error! Perfect  │
│                  │  binary search required.     │
└──────────────────┴──────────────────────────────┘
```

---

## Related Features

This feature enables:
- **#2 Difficulty Presets** - Could show "This matches our Medium preset!"
- **#21 Performance Metrics** - Post-game, compare actual vs. predicted efficiency
- **#4 Daily Challenge** - Show difficulty of daily challenge
- **#3 Hot/Cold Hints** - Adjust difficulty if hints are enabled

---

## Files to Create/Modify

### New Files
1. `src/difficulty.rs` - Difficulty calculation logic (~200 lines)
2. `templates/difficulty_indicator.html` - Askama template (~40 lines)

### Modified Files
1. `src/lib.rs` - Export difficulty module (1 line)
2. `src/templates.rs` - Add DifficultyIndicator struct (~10 lines)
3. `src/web.rs` - Add `/difficulty-preview` endpoint (~30 lines)
4. `static/index.html` - HTMX triggers + CSS (~100 lines)

**Total**: ~380 lines of code

---

**Priority**: ⭐⭐⭐⭐⭐ (5/5)
**Impact**: High (UX, Education)
**Effort**: Low (1-2 days)
**Risk**: Very Low (no breaking changes)
**Architecture**: ✅ Maintains separation of concerns
**Testing**: ✅ Unit + Integration testable

**Recommendation**: Implement this first! Great ROI, teaches users about binary search, and sets the tone for quality UX.
