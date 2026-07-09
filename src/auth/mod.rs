//! Authentication module.
//!
//! Provides extractors for authenticated users based on headers from oauth2-proxy.
//!
//! # Architecture
//!
//! This application runs behind oauth2-proxy, which handles all OAuth2/OIDC authentication
//! with Keycloak. When a request reaches the application:
//!
//! 1. oauth2-proxy intercepts the request
//! 2. If no valid session exists, redirects to Keycloak for login
//! 3. After successful authentication, oauth2-proxy forwards the request with headers:
//!    - `X-Forwarded-User`: User ID (OIDC subject)
//!    - `X-Forwarded-Email`: User's email address
//!    - `X-Forwarded-Preferred-Username`: Username
//!    - `X-Forwarded-Groups`: Comma-separated list of groups
//!
//! The application simply extracts these headers to identify the user.
//!
//! # Security
//!
//! The application trusts these headers because:
//! - The app is network-isolated (not exposed externally)
//! - All traffic must go through oauth2-proxy (enforced by docker networking)
//! - Direct connections to the app port are not possible from outside

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// Authenticated user extracted from oauth2-proxy headers.
///
/// All routes require authentication, so this extractor will always succeed
/// (or return 401 if headers are missing, indicating misconfiguration).
///
/// # Example
///
/// ```rust
/// use axum::extract::State;
/// use number_guessing_game::auth::AuthenticatedUser;
///
/// async fn my_handler(
///     user: AuthenticatedUser,
///     State(pool): State<PgPool>,
/// ) -> String {
///     format!("Hello, {}!", user.email)
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    /// Unique user identifier (OIDC subject)
    ///
    /// This is a stable identifier that won't change even if the user
    /// updates their email or username.
    pub user_id: String,

    /// User's email address
    pub email: String,

    /// Preferred username (may be different from email)
    pub username: Option<String>,

    /// Groups the user belongs to (e.g., "admin")
    ///
    /// Can be used for authorization checks.
    pub groups: Vec<String>,
}

impl AuthenticatedUser {
    /// Check if user is a member of the specified group.
    ///
    /// # Example
    ///
    /// ```rust
    /// if user.is_in_group("admin") {
    ///     // User is an admin
    /// }
    /// ```
    pub fn is_in_group(&self, group: &str) -> bool {
        self.groups.iter().any(|g| g == group)
    }

    /// Check if user is an administrator.
    pub fn is_admin(&self) -> bool {
        self.is_in_group("admin")
    }
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let headers = &parts.headers;

        // Extract user ID (required)
        // oauth2-proxy in proxy mode sends X-Forwarded-* headers, not X-Auth-Request-*
        let user_id = headers
            .get("X-Forwarded-User")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                warn!("Missing X-Forwarded-User header - oauth2-proxy misconfigured?");
                (
                    StatusCode::UNAUTHORIZED,
                    "Missing authentication headers - please contact support",
                )
            })?
            .to_string();

        // Extract email (required)
        let email = headers
            .get("X-Forwarded-Email")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                warn!(
                    user_id = %user_id,
                    "Missing X-Forwarded-Email header - oauth2-proxy misconfigured?"
                );
                (
                    StatusCode::UNAUTHORIZED,
                    "Missing authentication headers - please contact support",
                )
            })?
            .to_string();

        // Extract username (optional)
        let username = headers
            .get("X-Forwarded-Preferred-Username")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        // Extract groups (optional, comma-separated)
        let groups = headers
            .get("X-Forwarded-Groups")
            .and_then(|v| v.to_str().ok())
            .map(|s| {
                s.split(',')
                    .map(|g| g.trim().to_string())
                    .filter(|g| !g.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        debug!(
            user_id = %user_id,
            email = %email,
            username = ?username,
            groups = ?groups,
            uri = %parts.uri,
            "Authenticated user extracted from headers"
        );

        Ok(AuthenticatedUser {
            user_id,
            email,
            username,
            groups,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_in_group() {
        let user = AuthenticatedUser {
            user_id: "123".to_string(),
            email: "test@example.com".to_string(),
            username: Some("test".to_string()),
            groups: vec!["admin".to_string(), "users".to_string()],
        };

        assert!(user.is_in_group("admin"));
        assert!(user.is_in_group("users"));
        assert!(!user.is_in_group("moderators"));
    }

    #[test]
    fn test_is_admin() {
        let admin_user = AuthenticatedUser {
            user_id: "123".to_string(),
            email: "admin@example.com".to_string(),
            username: Some("admin".to_string()),
            groups: vec!["admin".to_string()],
        };

        let regular_user = AuthenticatedUser {
            user_id: "456".to_string(),
            email: "user@example.com".to_string(),
            username: Some("user".to_string()),
            groups: vec!["users".to_string()],
        };

        assert!(admin_user.is_admin());
        assert!(!regular_user.is_admin());
    }
}
