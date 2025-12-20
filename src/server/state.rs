use axum::extract::FromRef;
use axum_csrf::CsrfConfig;
use crate::db::GameRepository;

/// Application state shared across all handlers.
///
/// This struct holds the repository implementation and is cloned
/// for each request handler. The generic parameter allows for
/// different repository implementations (PostgreSQL, in-memory, etc.)
/// with static dispatch for zero runtime overhead.
///
/// # Type Parameters
/// * `R` - The repository implementation, must implement `GameRepository`
///
/// # Cloning
/// The struct derives Clone because repository implementations must
/// be Clone (trait bound). This allows Axum to share state across handlers.
#[derive(Clone)]
pub struct AppState<R: GameRepository> {
    /// The game repository instance
    pub repo: R,
    /// CSRF configuration
    pub csrf_config: CsrfConfig,
}

impl<R: GameRepository> AppState<R> {
    /// Create a new application state with the given repository and CSRF config
    pub fn new(repo: R, csrf_config: CsrfConfig) -> Self {
        Self { repo, csrf_config }
    }
}

// Required for axum-csrf to work with combined state
impl<R: GameRepository> FromRef<AppState<R>> for CsrfConfig {
    fn from_ref(state: &AppState<R>) -> Self {
        state.csrf_config.clone()
    }
}