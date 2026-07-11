//! Askama templates for HTML responses.
//!
//! This module contains template structs that render HTML using the Askama template engine.
//! Templates are type-safe and compiled at build time.

use crate::core::GameId;
use crate::core::features::difficulty::DifficultyInfo;
use askama::Template;
use askama_web::WebTemplate;

/// Template for the main index page
#[derive(Template, WebTemplate)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub csrf_token: String,
}

/// Template for displaying error messages
#[derive(Template, WebTemplate)]
#[template(path = "error.html")]
pub struct ErrorTemplate<'a> {
    pub error_message: &'a str,
}

/// Template for game initialization screen
#[derive(Template, WebTemplate)]
#[template(path = "game_started.html")]
pub struct GameStartedTemplate {
    pub game_id: GameId,
    pub min: i32,
    pub max: i32,
    pub csrf_token: String,
    /// Counter pill state (precomputed; false/empty when the game is unlimited)
    pub show_counter: bool,
    pub counter_label: String,
    /// Dots in the counter pill; 0 when the limit is over 10 (label only)
    pub dots_unspent: u32,
}

/// Template for guess form with feedback
///
/// Rendered into `#game-content` after each in-progress guess. Also carries
/// HTMX out-of-band fragments that update the range tracker (`#track-live`,
/// `#track-marks`, `#track-labels`) and replace the guesses counter
/// (`#counter`).
#[derive(Template, WebTemplate)]
#[template(path = "guess_form.html")]
pub struct GuessFormTemplate {
    pub game_id: GameId,
    pub min: i32,
    pub max: i32,
    /// Sanitized display window after this guess (drives hidden form fields
    /// and the emphasized bound labels on the tracker).
    pub low: i32,
    pub high: i32,
    /// Whether each side has narrowed past the original range limit; controls
    /// which label (range end vs closest guess) renders emphasized.
    pub has_low_bound: bool,
    pub has_high_bound: bool,
    /// Bound-label positions (percent), spread apart when the window is
    /// tight so the numbers never overlap.
    pub low_label_pct: u8,
    pub high_label_pct: u8,
    /// Tracker geometry in percent of the full range.
    pub live_left: u8,
    pub live_width: u8,
    pub mark_pos: u8,
    /// Counter pill state (precomputed; false/empty when the game is unlimited)
    pub show_counter: bool,
    pub counter_class: String,
    pub counter_label: String,
    /// Dots in the counter pill; both 0 when the limit is over 10 (label only)
    pub dots_spent: u32,
    pub dots_unspent: u32,
    pub feedback_class: String,
    pub feedback_message: String,
    pub csrf_token: String,
}

/// Template for game completion (win or lose)
#[derive(Template, WebTemplate)]
#[template(path = "game_complete.html")]
pub struct GameCompleteTemplate {
    pub win: bool,
    /// Win: "Found in N guesses — optimal is M." / Lose: "Out of guesses …"
    pub message: String,
    pub number: i32,
    /// Position of the number on the tracker (percent); anchors the
    /// magnifier cone from the bar to the halo on a win.
    pub mark_pos: u8,
    /// Tangent points of the magnifier cone on the halo circle, in
    /// zoom-cone viewBox units (win only; empty strings on a loss).
    pub t1x: String,
    pub t1y: String,
    pub t2x: String,
    pub t2y: String,
    /// Whether the game had a guess limit (controls counter cleanup OOB).
    pub had_limit: bool,
}

/// Template for game not found error
#[derive(Template, WebTemplate)]
#[template(path = "game_not_found.html")]
pub struct GameNotFoundTemplate;

/// Template for update error
#[derive(Template, WebTemplate)]
#[template(path = "update_error.html")]
pub struct UpdateErrorTemplate;

/// Template for difficulty indicator preview
#[derive(Template, WebTemplate)]
#[template(path = "difficulty_indicator.html")]
pub struct DifficultyIndicator {
    pub info: DifficultyInfo,
}
