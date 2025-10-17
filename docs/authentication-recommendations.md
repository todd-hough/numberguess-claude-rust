# Authentication Recommendations for Number Guessing Game

## Executive Summary

This document provides recommendations for adding authentication to the Number Guessing Game using modern OAuth2/OIDC standards. Based on research conducted in January 2025, the recommended approach is to use **OpenID Connect (OIDC)** with **PKCE** for all client types, leveraging Rust's mature authentication ecosystem.

## ✅ Implemented Solution (January 2025)

**Architecture:** Authentication Proxy Pattern with oauth2-proxy + Keycloak

The application now uses an authentication proxy pattern instead of embedding authentication directly in the Rust application. This provides several benefits:

### Architecture Overview

```
┌─────────────┐         ┌──────────────┐         ┌─────────────┐         ┌──────────┐
│   Browser   │────────>│ oauth2-proxy │────────>│     App     │────────>│PostgreSQL│
│   (User)    │<────────│  (Port 8080) │<────────│ (Port 4080) │<────────│          │
└─────────────┘         └──────────────┘         └─────────────┘         └──────────┘
      │                        │
      │                        │
      │                        ▼
      │                 ┌──────────────┐
      │                 │   Keycloak   │
      └────────────────>│ (Port 8090)  │
                        └──────────────┘
                               │
                               ▼
                        ┌──────────────┐
                        │    Redis     │
                        │  (Sessions)  │
                        └──────────────┘
```

### Components

1. **oauth2-proxy (Port 8080)**: Authentication gateway
   - External access point
   - Handles OAuth2/OIDC flow
   - Validates sessions stored in Redis
   - Forwards authenticated requests to app with user headers
   - PKCE with S256 enabled

2. **Keycloak (Port 8090)**: OIDC Identity Provider
   - Self-hosted, open-source
   - User management and authentication
   - Group-based authorization
   - Configured with `numberguess` realm
   - Default user: admin@local.test / password

3. **Redis**: Session Storage
   - Stores oauth2-proxy sessions
   - Enables horizontal scaling
   - Automatic session expiration

4. **Application (Port 4080)**: Internal only
   - Not exposed externally
   - Receives authenticated requests with headers:
     - `X-Auth-Request-User`: User ID (OIDC subject)
     - `X-Auth-Request-Email`: User's email
     - `X-Auth-Request-Preferred-Username`: Username
     - `X-Auth-Request-Groups`: Comma-separated groups
   - Uses `AuthenticatedUser` extractor in handlers

### Why This Approach?

**Advantages over embedded authentication:**
- ✅ **Zero OAuth libraries in Rust code**: No external Rust dependencies for auth
- ✅ **Language-agnostic**: Pattern works with any backend language/framework
- ✅ **Network isolation**: Application not exposed directly to internet
- ✅ **Separation of concerns**: Authentication completely separate from business logic
- ✅ **Easy to swap providers**: Change Keycloak to any OIDC provider
- ✅ **No code changes for auth updates**: Update oauth2-proxy/Keycloak independently
- ✅ **Horizontal scaling**: Redis-backed sessions support multiple app instances

**Trade-offs:**
- ❌ More complex Docker Compose setup (4 additional services)
- ❌ Cannot test auth without full stack running
- ❌ Additional network hops (minimal latency impact)

### Implementation Files

- `src/auth/mod.rs`: `AuthenticatedUser` extractor (reads oauth2-proxy headers)
- `docker-compose.yml`: Full authentication stack configuration
- `keycloak/realm-export.json`: Keycloak realm with users and groups
- `.env`: OAuth2 secrets (OAUTH2_COOKIE_SECRET, OAUTH2_CLIENT_SECRET)

### Usage

```bash
# Start full stack with authentication
make dev

# Access at http://localhost:8080
# Login with admin@local.test / password
```

### Security Features

- **PKCE with S256**: OAuth2 authorization code flow protection
- **Network isolation**: App on internal port 4080, not accessible externally
- **Session security**: Redis-backed sessions, httponly cookies, samesite=lax
- **Group-based authorization**: `AuthenticatedUser::is_in_group()` and `is_admin()`
- **All routes protected**: No anonymous access to web interface

### Configuration

Environment variables in `.env`:
```bash
# OAuth2 Secrets (generate with: openssl rand -base64 32)
OAUTH2_COOKIE_SECRET=<32-byte-base64-secret>
OAUTH2_CLIENT_SECRET=<keycloak-client-secret>
```

Docker Compose services:
- `postgres`: PostgreSQL database
- `redis`: Session storage
- `keycloak`: OIDC identity provider
- `oauth2-proxy`: Authentication gateway
- `app`: Number guessing game application

### Alternative Approaches Considered

The original recommendations below explore **axum-oidc** and **tower-sessions** for embedding authentication directly in the Rust application. While this is a valid approach, the authentication proxy pattern was chosen for:
1. Simpler Rust code (no auth dependencies)
2. Better separation of concerns
3. Network isolation
4. Easier to maintain and update

For projects requiring embedded authentication (e.g., no Docker, single binary deployment), see the recommendations below.

---

## Standards Overview

### OAuth 2.0 and OAuth 2.1

**OAuth 2.0** is the industry-standard authorization framework that allows third-party applications to obtain limited access to user accounts.

**OAuth 2.1** (upcoming standard) consolidates OAuth 2.0 with modern security best practices:
- **Requires PKCE** for all authorization code flows (public AND confidential clients)
- **Removes implicit flow** (deprecated due to security concerns)
- **Removes resource owner password credentials** grant
- Incorporates security guidelines from RFC 9700 (January 2025)

### OpenID Connect (OIDC)

**OIDC** builds on top of OAuth 2.0, adding an **identity layer**:
- OAuth 2.0 = Authorization ("what can this app do?")
- OIDC = Authentication ("who is this user?") + Authorization

OIDC provides:
- Standardized user identity claims (email, name, profile)
- ID tokens (JWT) for user information
- UserInfo endpoint for additional profile data
- Discovery mechanism for provider configuration
- Support for social logins (Google, GitHub, Microsoft, etc.)

### PKCE (Proof Key for Code Exchange)

**Current Best Practice (RFC 9700, Jan 2025):**
- **REQUIRED** for public clients (browser-based, mobile apps)
- **RECOMMENDED** for confidential clients (web apps with backend)
- **REQUIRED** in OAuth 2.1 for all clients

PKCE prevents:
- Authorization code interception attacks
- CSRF attacks
- Token replay attacks

## Recommended Architecture

### Option 1: OIDC with Third-Party Provider (Recommended)

**Best for:** Most use cases, especially if you want social login support

```
┌─────────────┐         ┌──────────────┐         ┌─────────────────┐
│   Browser   │────────>│  Your Axum   │────────>│  OIDC Provider  │
│   (User)    │<────────│    Server    │<────────│ (Google, etc.)  │
└─────────────┘         └──────────────┘         └─────────────────┘
      │                        │
      │    Session Cookie      │
      │                        │
      └────────────────────────┘
                          │
                   ┌──────▼──────┐
                   │  PostgreSQL │
                   │  (Sessions) │
                   └─────────────┘
```

**Pros:**
- No password management on your side
- Users can use existing accounts (Google, GitHub, Microsoft, etc.)
- Provider handles security updates, MFA, account recovery
- Faster implementation
- Industry-standard security

**Cons:**
- Dependency on external provider
- Requires internet connectivity
- Users must have account with provider

### Option 2: Self-Hosted OAuth2 Server

**Best for:** Enterprise deployments, custom requirements, air-gapped environments

**Pros:**
- Complete control over authentication
- No external dependencies
- Custom branding and flows
- Can work offline

**Cons:**
- More complex to implement and maintain
- Responsible for security updates
- Must handle password storage, MFA, account recovery
- Higher development and maintenance cost

## Recommended Rust Libraries

### Primary Stack (OIDC with Third-Party Provider)

#### 1. **axum-oidc** (v0.6.0+)
```toml
axum-oidc = "0.6"
```

**Purpose:** High-level OIDC integration for Axum

**Features:**
- Ready-to-use OIDC authentication middleware
- Two middleware layers:
  - `OidcAuthLayer`: Optional authentication (check if user is logged in)
  - `OidcLoginLayer`: Enforced authentication (redirect if not logged in)
- Built-in extractors:
  - `OidcClaims`: Get user identity claims (email, name, etc.)
  - `OidcAccessToken`: Get access token for API calls
  - `OidcRpInitiatedLogout`: Get logout URL
- Works with any OIDC-compliant provider

**Example:**
```rust
use axum_oidc::{OidcAuthLayer, OidcLoginLayer, OidcClaims};

// Protected route - requires authentication
async fn protected_handler(claims: OidcClaims) -> String {
    format!("Hello, {}!", claims.subject)
}

// Optional authentication
async fn optional_handler(claims: Option<OidcClaims>) -> String {
    match claims {
        Some(c) => format!("Hello, {}!", c.subject),
        None => "Hello, guest!".to_string(),
    }
}
```

**GitHub:** https://github.com/pfzetto/axum-oidc

#### 2. **tower-sessions** + **tower-sessions-sqlx-store**
```toml
tower-sessions = "0.14"
tower-sessions-sqlx-store = "0.14"
```

**Purpose:** Session management with PostgreSQL backend

**Features:**
- Tower middleware for sessions (works seamlessly with Axum)
- PostgreSQL session persistence (you're already using PostgreSQL!)
- Configurable session lifetime
- Secure cookie management
- Works with your existing PgPool

**Example:**
```rust
use tower_sessions::{SessionManagerLayer, Expiry};
use tower_sessions_sqlx_store::PostgresStore;
use sqlx::PgPool;

let pool = PgPool::connect(&database_url).await?;
let session_store = PostgresStore::new(pool.clone());
session_store.migrate().await?; // Creates session table

let session_layer = SessionManagerLayer::new(session_store)
    .with_expiry(Expiry::OnInactivity(time::Duration::days(7)));

let app = Router::new()
    .route("/", get(handler))
    .layer(session_layer);
```

**Note:** tower-sessions replaced the older `axum-sessions` due to bugs and design flaws. Use tower-sessions for new projects.

#### 3. **openidconnect** (Low-Level, Optional)
```toml
openidconnect = "3.5"
```

**Purpose:** If you need more control than axum-oidc provides

**Features:**
- Full OpenID Connect implementation
- Strongly-typed API
- PKCE support (S256 method recommended)
- Token validation and refresh
- Works with async and sync HTTP clients

**Use when:**
- axum-oidc doesn't support your use case
- You need custom OIDC flows
- You're building a custom authentication system

**GitHub:** https://github.com/ramosbugs/openidconnect-rs

### Alternative Stack (OAuth2 Only, More Control)

#### **oauth2** + **tower-sessions**
```toml
oauth2 = "5.0"
tower-sessions = "0.14"
tower-sessions-sqlx-store = "0.14"
```

**Use when:**
- You only need OAuth2 (authorization), not OIDC (authentication)
- You want maximum control over the flow
- You're integrating with non-OIDC OAuth2 providers

**oauth2 crate features:**
- Full OAuth2 RFC 6749 implementation
- PKCE support (S256 recommended)
- Strongly-typed API
- Framework-agnostic
- Custom HTTP client support (works with reqwest, curl, etc.)
- MSRV: Rust 1.65

**GitHub:** https://github.com/ramosbugs/oauth2-rs

## Recommended OIDC Providers

### Free/Developer-Friendly Options

1. **Google Identity Platform**
   - Free tier: Unlimited authentications
   - Easy setup, well-documented
   - Social login: Google accounts
   - Best for: Public-facing apps

2. **GitHub OAuth**
   - Free for public repositories
   - Developer-focused audience
   - Social login: GitHub accounts
   - Best for: Developer tools

3. **Auth0** (by Okta)
   - Free tier: 7,000 active users/month
   - Supports multiple social providers
   - Excellent documentation and SDKs
   - Best for: Multi-provider support

4. **Microsoft Entra ID** (formerly Azure AD)
   - Free tier: 50,000 monthly active users
   - Enterprise-grade
   - Supports Microsoft, Google, Facebook
   - Best for: Enterprise apps

5. **Keycloak** (Self-Hosted)
   - Open source
   - Self-hosted (run in Docker)
   - Complete control
   - Best for: Self-hosted, enterprise deployments

### Provider Comparison

| Provider | Free Tier | Social Logins | OIDC | Setup Difficulty | Best For |
|----------|-----------|---------------|------|------------------|----------|
| Google | Unlimited | Google only | Yes | Easy | Simple apps |
| GitHub | Unlimited | GitHub only | No (OAuth2) | Easy | Dev tools |
| Auth0 | 7k MAU | Multiple | Yes | Medium | Production apps |
| Microsoft | 50k MAU | Multiple | Yes | Medium | Enterprise |
| Keycloak | Self-hosted | Multiple | Yes | Hard | Self-hosted |

**Recommendation:** Start with **Google** for simplicity, or **Auth0** if you want multi-provider support.

## Implementation Approach

### Phase 1: Add Session Management

**Goal:** Set up session infrastructure that will be used for authentication

**Steps:**
1. Add dependencies to `Cargo.toml`:
   ```toml
   tower-sessions = "0.14"
   tower-sessions-sqlx-store = "0.14"
   time = "0.3" # For session expiry
   ```

2. Create session migration:
   ```sql
   -- migrations/YYYYMMDDHHMMSS_create_sessions_table.sql
   CREATE TABLE IF NOT EXISTS tower_sessions (
       id TEXT PRIMARY KEY NOT NULL,
       data BYTEA NOT NULL,
       expiry_date TIMESTAMPTZ NOT NULL
   );

   CREATE INDEX IF NOT EXISTS tower_sessions_expiry_date_idx
   ON tower_sessions (expiry_date);
   ```

3. Update `src/server/mod.rs`:
   ```rust
   use tower_sessions::{SessionManagerLayer, Expiry};
   use tower_sessions_sqlx_store::PostgresStore;

   pub async fn run_server(pool: PgPool, port: u16) {
       // Create session store
       let session_store = PostgresStore::new(pool.clone());
       session_store.migrate().await.expect("Failed to migrate sessions");

       // Session layer with 7-day inactivity timeout
       let session_layer = SessionManagerLayer::new(session_store)
           .with_expiry(Expiry::OnInactivity(time::Duration::days(7)));

       let app = Router::new()
           .nest("/api", api_routes)
           .merge(web_routes)
           .fallback_service(ServeDir::new("static"))
           .layer(session_layer) // Add session layer
           .layer(TraceLayer::new_for_http());

       // ... rest of server setup
   }
   ```

**Result:** Sessions are now persisted in PostgreSQL, with automatic cleanup of expired sessions.

### Phase 2: Integrate OIDC Authentication

**Goal:** Add OIDC authentication with a provider (e.g., Google)

**Steps:**

1. Register your application with the OIDC provider:
   - **Google**: https://console.cloud.google.com/apis/credentials
   - Set redirect URI: `http://localhost:8080/auth/callback` (dev) or `https://yourdomain.com/auth/callback` (prod)
   - Note down: **Client ID** and **Client Secret**

2. Add environment variables to `.env`:
   ```bash
   # OIDC Configuration
   OIDC_ISSUER_URL=https://accounts.google.com
   OIDC_CLIENT_ID=your-client-id
   OIDC_CLIENT_SECRET=your-client-secret
   OIDC_REDIRECT_URL=http://localhost:8080/auth/callback
   ```

3. Add dependencies:
   ```toml
   axum-oidc = "0.6"
   ```

4. Create authentication module `src/auth/mod.rs`:
   ```rust
   use axum_oidc::{OidcClient, OidcClientConfig};

   pub fn create_oidc_client() -> OidcClient {
       let issuer_url = std::env::var("OIDC_ISSUER_URL")
           .expect("OIDC_ISSUER_URL must be set");
       let client_id = std::env::var("OIDC_CLIENT_ID")
           .expect("OIDC_CLIENT_ID must be set");
       let client_secret = std::env::var("OIDC_CLIENT_SECRET")
           .expect("OIDC_CLIENT_SECRET must be set");
       let redirect_url = std::env::var("OIDC_REDIRECT_URL")
           .expect("OIDC_REDIRECT_URL must be set");

       OidcClientConfig::new()
           .issuer_url(issuer_url)
           .client_id(client_id)
           .client_secret(client_secret)
           .redirect_url(redirect_url)
           .build()
           .expect("Failed to create OIDC client")
   }
   ```

5. Update routing in `src/server/mod.rs`:
   ```rust
   use axum_oidc::{OidcAuthLayer, OidcLoginLayer};
   use crate::auth::create_oidc_client;

   pub async fn run_server(pool: PgPool, port: u16) {
       let oidc_client = create_oidc_client();

       // Public routes (no auth required)
       let public_routes = Router::new()
           .route("/", get(landing_page))
           .route("/health", get(health_check));

       // Protected routes (auth required)
       let protected_routes = Router::new()
           .route("/game/new", post(create_game_web))
           .route("/game/{game_id}/guess", post(make_guess_web))
           .layer(OidcLoginLayer::new(oidc_client.clone()))
           .with_state(pool.clone());

       // API routes (optional auth)
       let api_routes = Router::new()
           .route("/games", post(create_game_api))
           .route("/games/{game_id}/guess", post(make_guess_api))
           .layer(OidcAuthLayer::new(oidc_client.clone()))
           .with_state(pool.clone());

       let app = Router::new()
           .merge(public_routes)
           .merge(protected_routes)
           .nest("/api", api_routes)
           .fallback_service(ServeDir::new("static"))
           .layer(session_layer);

       // ... rest of server setup
   }
   ```

6. Update handlers to use user identity:
   ```rust
   use axum_oidc::OidcClaims;

   // Extract user info from OIDC claims
   pub async fn create_game_web(
       State(pool): State<PgPool>,
       claims: OidcClaims, // Extracts authenticated user
       Form(payload): Form<CreateGameRequest>,
   ) -> impl IntoResponse {
       let user_email = claims.email.as_ref().unwrap();
       info!(user = %user_email, "Creating game for authenticated user");

       // ... rest of handler
   }
   ```

### Phase 3: Add User Management (Optional)

**Goal:** Store user information and associate games with users

**Steps:**

1. Create users table migration:
   ```sql
   -- migrations/YYYYMMDDHHMMSS_create_users_table.sql
   CREATE TABLE IF NOT EXISTS users (
       user_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       oidc_subject TEXT UNIQUE NOT NULL,
       email TEXT NOT NULL,
       name TEXT,
       created_at TIMESTAMPTZ DEFAULT NOW(),
       last_login_at TIMESTAMPTZ DEFAULT NOW()
   );

   -- Add user_id to games table
   ALTER TABLE games ADD COLUMN user_id UUID REFERENCES users(user_id) ON DELETE CASCADE;
   ```

2. Create user model `src/db/user.rs`:
   ```rust
   use sqlx::PgPool;
   use uuid::Uuid;

   pub async fn upsert_user(
       pool: &PgPool,
       oidc_subject: &str,
       email: &str,
       name: Option<&str>,
   ) -> Result<Uuid, sqlx::Error> {
       let row = sqlx::query!(
           r#"
           INSERT INTO users (oidc_subject, email, name)
           VALUES ($1, $2, $3)
           ON CONFLICT (oidc_subject)
           DO UPDATE SET
               email = EXCLUDED.email,
               name = EXCLUDED.name,
               last_login_at = NOW()
           RETURNING user_id
           "#,
           oidc_subject,
           email,
           name,
       )
       .fetch_one(pool)
       .await?;

       Ok(row.user_id)
   }
   ```

3. Update game handlers to link games to users:
   ```rust
   pub async fn create_game_web(
       State(pool): State<PgPool>,
       claims: OidcClaims,
       Form(payload): Form<CreateGameRequest>,
   ) -> impl IntoResponse {
       // Upsert user on each request (updates last_login_at)
       let user_id = upsert_user(
           &pool,
           &claims.subject,
           claims.email.as_ref().unwrap(),
           claims.name.as_deref(),
       )
       .await
       .unwrap();

       // Create game with user_id
       let game_id = db::create_game_for_user(
           &pool,
           user_id,
           payload.min,
           payload.max,
           payload.max_guesses,
       )
       .await
       .unwrap();

       // ... return response
   }
   ```

### Phase 4: Add Features (Optional)

Consider adding:
- **Leaderboards:** Track best scores per user
- **Game history:** Show past games
- **User profiles:** Display user stats
- **Multiplayer:** Challenge other authenticated users
- **Account settings:** User preferences

## Security Considerations

### Critical Requirements

1. **Always use HTTPS in production**
   - OIDC requires secure redirect URIs
   - Protects session cookies from interception
   - Use Let's Encrypt for free SSL certificates

2. **Secure cookie settings**
   ```rust
   let session_layer = SessionManagerLayer::new(session_store)
       .with_secure(true)     // HTTPS only (prod)
       .with_http_only(true)  // Prevent JS access
       .with_same_site(SameSite::Lax); // CSRF protection
   ```

3. **Enable PKCE with S256**
   - axum-oidc enables this by default
   - If using oauth2 crate manually, ensure S256 challenge method

4. **Validate redirect URIs**
   - Only allow pre-registered redirect URIs
   - Never allow open redirects

5. **Token storage**
   - Store tokens server-side in sessions
   - Never expose tokens to client-side JavaScript
   - Refresh tokens securely

6. **Session management**
   - Set reasonable session timeouts (7-30 days)
   - Implement inactivity timeouts
   - Provide logout functionality
   - Clean up expired sessions (tower-sessions does this automatically)

### Best Practices (RFC 9700, Jan 2025)

1. **Use PKCE for all clients** (public and confidential)
2. **Avoid implicit flow** (use authorization code flow instead)
3. **Validate ID tokens** (signature, issuer, audience, expiry)
4. **Rotate refresh tokens** on each use
5. **Implement rate limiting** on auth endpoints
6. **Log authentication events** for audit trail
7. **Use short-lived access tokens** (15 minutes typical)

### OWASP Considerations

- **A01:2021 – Broken Access Control:** Validate user permissions on every request
- **A02:2021 – Cryptographic Failures:** Use HTTPS, secure cookies, token encryption
- **A03:2021 – Injection:** Use parameterized queries (you already do with SQLx)
- **A05:2021 – Security Misconfiguration:** Review OIDC client configuration
- **A07:2021 – Identification and Authentication Failures:** OIDC handles this, but implement proper session management

## Migration Strategy

### For Existing Users

If you have existing users (currently your app has no users), plan the migration:

1. **Greenfield (Recommended):**
   - Start fresh with authentication
   - All new users authenticate via OIDC
   - No migration needed

2. **Account Linking:**
   - Allow users to link existing accounts to OIDC
   - Email matching for automatic linking
   - Manual verification process

### Database Schema Changes

Add these tables:
- `tower_sessions`: Session storage (managed by tower-sessions)
- `users`: User profiles (optional, for features like leaderboards)
- `games.user_id`: Foreign key to users (optional)

## Testing Considerations

### Unit Tests
- Mock OIDC claims in tests
- Test authorization logic independently
- Verify token validation

### Integration Tests
- Use test OIDC provider (e.g., mock server)
- Test complete authentication flow
- Verify session persistence

### Example Test Helper
```rust
#[cfg(test)]
mod test_helpers {
    use axum_oidc::OidcClaims;

    pub fn mock_claims(email: &str) -> OidcClaims {
        OidcClaims {
            subject: "test-user-123".to_string(),
            email: Some(email.to_string()),
            name: Some("Test User".to_string()),
            // ... other fields
        }
    }
}
```

## Cost Estimation

### Free Tier Comparison (as of Jan 2025)

- **Google:** Unlimited authentications (free)
- **GitHub:** Unlimited (free for public repos)
- **Auth0:** 7,000 MAU (monthly active users)
- **Microsoft Entra ID:** 50,000 MAU
- **Keycloak:** Self-hosted (server costs only)

**For your use case:** All providers' free tiers are likely sufficient unless you have >7,000 active users/month.

## Recommended Next Steps

1. **Decide on provider:** Start with Google for simplicity
2. **Implement Phase 1:** Add session management (no breaking changes)
3. **Register OIDC application:** Get client credentials
4. **Implement Phase 2:** Add OIDC authentication
5. **Test thoroughly:** Verify login/logout flows
6. **Update documentation:** Document auth requirements for API users
7. **Deploy gradually:** Feature flag for gradual rollout

## Additional Resources

### Documentation
- [RFC 9700 - OAuth 2.0 Security Best Practices](https://datatracker.ietf.org/doc/rfc9700/)
- [OpenID Connect Core Spec](https://openid.net/specs/openid-connect-core-1_0.html)
- [axum-oidc GitHub](https://github.com/pfzetto/axum-oidc)
- [tower-sessions GitHub](https://github.com/maxcountryman/tower-sessions)
- [oauth2-rs GitHub](https://github.com/ramosbugs/oauth2-rs)

### Tutorials
- [Shuttle.dev: OAuth with Axum](https://www.shuttle.dev/blog/2023/08/30/using-oauth-with-axum)
- [LogRocket: JWT Authentication with Rust and Axum](https://blog.logrocket.com/using-rust-axum-build-jwt-authentication-api/)

### Example Projects
- Check `examples/` folder in axum-oidc repository
- Search GitHub for "axum oidc" for real-world examples

## Questions to Consider

Before implementing, decide:

1. **Who should authenticate?**
   - All users? (recommended for accountability)
   - Only for certain features? (e.g., leaderboards)
   - Optional? (guest mode + authenticated mode)

2. **Which providers to support?**
   - Single provider (Google) for simplicity?
   - Multiple providers (Google + GitHub) for flexibility?
   - Self-hosted (Keycloak) for control?

3. **What user data to store?**
   - Minimal (just OIDC subject)?
   - Profile info (name, email)?
   - Game history and stats?

4. **How to handle existing functionality?**
   - Require auth for all games?
   - Keep anonymous games, add authenticated games?
   - Gradual migration with feature flags?

## Conclusion

**Recommended Stack:**
- **axum-oidc** for OIDC authentication
- **tower-sessions + tower-sessions-sqlx-store** for session management
- **Google** as initial OIDC provider (easy to add more later)
- **PostgreSQL** for session and user storage (you're already using it!)

**Timeline Estimate:**
- Phase 1 (Sessions): 2-4 hours
- Phase 2 (OIDC): 4-8 hours
- Phase 3 (User Management): 4-6 hours (optional)
- Testing and documentation: 4-6 hours
- **Total:** 2-3 days for complete implementation

**Benefits:**
- Industry-standard security
- No password management burden
- Easy to add social login providers
- Scales with your application
- Leverages existing PostgreSQL infrastructure

This approach provides a solid foundation for authentication that can grow with your application's needs.
