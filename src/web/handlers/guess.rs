//! Web UI handler for guess processing.
//!
//! Processes player guesses via HTML forms (HTMX).

use crate::auth::AuthenticatedUser;
use crate::core::features::difficulty::calculate_optimal_guesses;
use crate::core::{GameId, GuessResult, GuessingGame};
use crate::db::{DbError, GameRepository};
use crate::server::state::AppState;
use crate::web::error::WebError;
use crate::web::templates::{GameCompleteTemplate, GuessFormTemplate};
use crate::web::types::MakeGuessRequest;
use axum::{
    extract::{Form, Path, State},
    response::{IntoResponse, Response},
};
use axum_csrf::CsrfToken;
use tracing::{debug, error, info, warn};

/// Web UI handler for making a guess (HTML).
///
/// Processes a guess and returns HTML response for HTMX.
/// Requires authentication via oauth2-proxy.
///
/// # Type Parameters
/// * `R` - The repository implementation (static dispatch for zero overhead)
pub async fn make_guess_web<R: GameRepository>(
    token: CsrfToken,
    State(state): State<AppState<R>>,
    user: AuthenticatedUser,
    Path(game_id): Path<GameId>,
    Form(payload): Form<MakeGuessRequest>,
) -> Result<Response, WebError> {
    // Verify CSRF token
    if token.verify(&payload.authenticity_token).is_err() {
        warn!("Web: CSRF token verification failed");
        return Err(WebError::InvalidCsrf);
    }

    debug!(
        user_id = %user.user_id,
        user_email = %user.email,
        game_id = %game_id,
        guess = payload.guess,
        "Web: Processing guess"
    );

    // Make guess using transactional approach (concurrency-safe). The
    // post-guess state comes back from the same transaction, so no follow-up
    // fetch is needed (a concurrent request could delete the game between
    // two calls).
    let (result, game) = state
        .repo
        .make_guess(game_id, payload.guess)
        .await
        .map_err(|e| match e {
            DbError::NotFound => {
                warn!(
                    game_id = %game_id,
                    "Web: Guess failed - game not found"
                );
                WebError::GameNotFound
            }
            e => {
                error!(
                    game_id = %game_id,
                    guess = payload.guess,
                    error = %e,
                    "Web: Failed to process guess"
                );
                WebError::UpdateFailed
            }
        })?;

    match result {
        GuessResult::TooLow => Ok(render_guess_form(
            token,
            game_id,
            &game,
            &payload,
            "too_low",
            "too-low",
            format!("{} is too low — aim higher.", payload.guess),
        )),
        GuessResult::TooHigh => Ok(render_guess_form(
            token,
            game_id,
            &game,
            &payload,
            "too_high",
            "too-high",
            format!("{} is too high — aim lower.", payload.guess),
        )),
        GuessResult::Correct { number, attempts } => {
            info!(
                user_id = %user.user_id,
                user_email = %user.email,
                game_id = %game_id,
                guess = payload.guess,
                number = number,
                attempts = attempts,
                result = "correct",
                "Web: Game completed - correct guess"
            );
            let (min, max) = game.range();
            let optimal = calculate_optimal_guesses(min, max);
            let guess_word = if attempts == 1 { "guess" } else { "guesses" };
            let mark_pos = range_pct(number, min, max);
            let ((t1x, t1y), (t2x, t2y)) = zoom_cone_tangents(mark_pos);
            let template = GameCompleteTemplate {
                win: true,
                message: format!("Found in {attempts} {guess_word} — optimal is {optimal}."),
                number,
                mark_pos,
                t1x: format!("{t1x:.1}"),
                t1y: format!("{t1y:.1}"),
                t2x: format!("{t2x:.1}"),
                t2y: format!("{t2y:.1}"),
                had_limit: game.max_guesses().is_some(),
            };
            Ok(template.into_response())
        }
        GuessResult::LimitReached {
            number,
            max_guesses,
        } => {
            info!(
                user_id = %user.user_id,
                user_email = %user.email,
                game_id = %game_id,
                guess = payload.guess,
                number = number,
                max_guesses = max_guesses,
                result = "limit_reached",
                "Web: Game completed - limit reached"
            );
            let (min, max) = game.range();
            let template = GameCompleteTemplate {
                win: false,
                message: format!("Out of guesses — you used all {max_guesses}."),
                number,
                mark_pos: range_pct(number, min, max),
                t1x: String::new(),
                t1y: String::new(),
                t2x: String::new(),
                t2y: String::new(),
                had_limit: true,
            };
            Ok(template.into_response())
        }
    }
}

/// Tangent points from the win mark on the tracker (the cone's apex) to the
/// halo circle, in zoom-cone viewBox units where 1 unit = 1% of card width.
/// The circle sits at center (50, 24) with radius 15 (halo is 30% wide and
/// pulled up so its center lands 24 units below the bar — see index.html).
/// The apex is always outside the circle (it is 24 units above the center,
/// radius is 15), so the tangent geometry never degenerates.
fn zoom_cone_tangents(mark_pos: u8) -> ((f64, f64), (f64, f64)) {
    const CX: f64 = 50.0;
    const CY: f64 = 24.0;
    const R: f64 = 15.0;
    let ux = f64::from(mark_pos) - CX;
    let uy = -CY;
    let d2 = ux * ux + uy * uy;
    let a = (R * R) / d2;
    let b = R * (d2 - R * R).sqrt() / d2;
    let t1 = (CX + a * ux - b * uy, CY + a * uy + b * ux);
    let t2 = (CX + a * ux + b * uy, CY + a * uy - b * ux);
    (t1, t2)
}

/// Position of `value` within `[min, max]` as a percentage (0-100).
/// A degenerate single-number range centers on the bar.
fn range_pct(value: i32, min: i32, max: i32) -> u8 {
    let span = i64::from(max) - i64::from(min);
    if span <= 0 {
        return 50;
    }
    let offset = i64::from(value.clamp(min, max)) - i64::from(min);
    ((offset * 100) / span).clamp(0, 100) as u8
}

/// Render the guess form for an ongoing game (too-low / too-high feedback),
/// using the post-guess state returned by the repository.
fn render_guess_form(
    token: CsrfToken,
    game_id: GameId,
    game: &GuessingGame,
    payload: &MakeGuessRequest,
    result_str: &str,
    feedback_class: &str,
    feedback_message: String,
) -> Response {
    let guess = payload.guess;
    debug!(
        game_id = %game_id,
        guess = guess,
        result = result_str,
        "Web: Guess result"
    );

    let (min, max) = game.range();
    let is_high = feedback_class == "too-high";

    // Sanitize the display-only tracker bounds from the form round-trip,
    // then narrow them by this guess. Cosmetic only: real validation is the
    // repository's job, so out-of-range or contradictory values just reset.
    //
    // Bounds are the nearest too-low / too-high GUESSES (inclusive), not the
    // remaining mathematical window: players recognize the numbers they
    // actually typed.
    let mut low = payload.low.unwrap_or(min).clamp(min, max);
    let mut high = payload.high.unwrap_or(max).clamp(min, max);
    if low > high {
        (low, high) = (min, max);
    }
    if is_high {
        high = high.min(guess).clamp(min, max);
    } else {
        low = low.max(guess).clamp(min, max);
    }
    if low > high {
        // Contradictory history (e.g. repeated guesses) - collapse for display
        high = low;
    }

    let live_left = range_pct(low, min, max);
    let high_pct = range_pct(high, min, max);
    let live_width = (high_pct - live_left).max(1);
    let has_low_bound = low > min;
    let has_high_bound = high < max;

    // Label positions: keep the two bound labels far enough apart to stay
    // legible when the window gets tight, and off the very ends of the bar.
    const LABEL_MIN_SEP: i32 = 14;
    const LABEL_EDGE: i32 = 4;
    let mut low_label = i32::from(live_left);
    let mut high_label = i32::from(high_pct);
    if has_low_bound && has_high_bound {
        // Spread a tight pair around its midpoint, then shift the pair back
        // inside the bar if the spread pushed it past an edge
        if high_label - low_label < LABEL_MIN_SEP {
            let mid = (low_label + high_label) / 2;
            low_label = mid - LABEL_MIN_SEP / 2;
            high_label = low_label + LABEL_MIN_SEP;
        }
        if low_label < LABEL_EDGE {
            high_label += LABEL_EDGE - low_label;
            low_label = LABEL_EDGE;
        }
        if high_label > 100 - LABEL_EDGE {
            low_label -= high_label - (100 - LABEL_EDGE);
            high_label = 100 - LABEL_EDGE;
        }
    }
    let low_label_pct = low_label.clamp(LABEL_EDGE, 100 - LABEL_EDGE) as u8;
    let high_label_pct = high_label.clamp(LABEL_EDGE, 100 - LABEL_EDGE) as u8;

    // Counter pill state (dots only render for limits of 10 or fewer)
    let remaining = game
        .max_guesses()
        .map(|limit| limit.saturating_sub(game.guess_count()));
    let (show_counter, counter_class, counter_label, dots_spent, dots_unspent) = match remaining {
        Some(r) if r > 0 => {
            let limit = game.max_guesses().unwrap_or(r);
            let class = match r {
                1 => "crit",
                2..=3 => "warn",
                _ => "",
            };
            let label = if r == 1 {
                "Last guess!".to_string()
            } else {
                format!("{r} left")
            };
            let (spent, unspent) = if limit <= 10 { (limit - r, r) } else { (0, 0) };
            (true, class, label, spent, unspent)
        }
        _ => (false, "", String::new(), 0, 0),
    };

    let csrf_token = token.authenticity_token().unwrap_or_default();
    let template = GuessFormTemplate {
        game_id,
        min,
        max,
        low,
        high,
        has_low_bound,
        has_high_bound,
        low_label_pct,
        high_label_pct,
        live_left,
        live_width,
        mark_pos: range_pct(guess, min, max),
        show_counter,
        counter_class: counter_class.to_string(),
        counter_label,
        dots_spent,
        dots_unspent,
        feedback_class: feedback_class.to_string(),
        feedback_message,
        csrf_token,
    };
    // Return token in tuple to trigger cookie setting via IntoResponseParts
    (token, template).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_pct_maps_bounds_and_centers_degenerate_ranges() {
        assert_eq!(range_pct(1, 1, 100), 0);
        assert_eq!(range_pct(100, 1, 100), 100);
        assert_eq!(range_pct(60, 0, 100), 60);
        // Values outside the range clamp to the bar
        assert_eq!(range_pct(-5, 0, 100), 0);
        assert_eq!(range_pct(200, 0, 100), 100);
        // Single-number range centers on the bar
        assert_eq!(range_pct(37, 37, 37), 50);
    }

    #[test]
    fn zoom_cone_tangent_points_lie_on_the_halo_circle() {
        for mark in [0u8, 4, 25, 50, 75, 96, 100] {
            let ((t1x, t1y), (t2x, t2y)) = zoom_cone_tangents(mark);
            let apex_x = f64::from(mark);
            for (tx, ty) in [(t1x, t1y), (t2x, t2y)] {
                // On the circle: |T - C| == R
                let (dx, dy) = (tx - 50.0, ty - 24.0);
                let dist = (dx * dx + dy * dy).sqrt();
                assert!((dist - 15.0).abs() < 1e-9, "mark {mark}: |T-C| = {dist}");
                // Tangent: radius C->T perpendicular to line T->apex
                let dot = dx * (tx - apex_x) + dy * ty;
                assert!(dot.abs() < 1e-9, "mark {mark}: not tangent (dot = {dot})");
                // Inside the zoom-cone viewBox (0 0 100 40)
                assert!((0.0..=100.0).contains(&tx), "mark {mark}: tx = {tx}");
                assert!((0.0..=40.0).contains(&ty), "mark {mark}: ty = {ty}");
            }
        }
    }
}
