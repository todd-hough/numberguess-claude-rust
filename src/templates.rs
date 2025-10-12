//! Askama templates for HTML responses.
//!
//! This module contains template structs that render HTML using the Askama template engine.
//! Templates are type-safe and compiled at build time.

use crate::game_id::GameId;
use askama::Template;

/// Template for displaying error messages
#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate<'a> {
    pub error_message: &'a str,
}

/// Template for game initialization screen
#[derive(Template)]
#[template(path = "game_started.html")]
pub struct GameStartedTemplate {
    pub game_id: GameId,
    pub min: i32,
    pub max: i32,
    pub max_guesses: Option<u32>,
}

/// Template for guess form with feedback
#[derive(Template)]
#[template(path = "guess_form.html")]
pub struct GuessFormTemplate {
    pub game_id: GameId,
    pub min: i32,
    pub max: i32,
    pub remaining_guesses: Option<u32>,
    pub feedback_class: String,
    pub feedback_message: String,
}

/// Template for game completion (win or lose)
#[derive(Template)]
#[template(path = "game_complete.html")]
pub struct GameCompleteTemplate {
    pub feedback_class: String,
    pub emoji: String,
    pub message: String,
    pub number: i32,
    pub attempts: Option<u32>,
}

/// Template for game not found error
#[derive(Template)]
#[template(path = "game_not_found.html")]
pub struct GameNotFoundTemplate;

/// Template for update error
#[derive(Template)]
#[template(path = "update_error.html")]
pub struct UpdateErrorTemplate;
