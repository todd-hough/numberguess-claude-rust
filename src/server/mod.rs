//! Server module.
//!
//! Handles server initialization, routing configuration, and startup.

use crate::api::handlers::{create_game_api, health_check, make_guess_api};
use crate::web::handlers::{create_game_web, difficulty_preview, make_guess_web};
use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;
use tower_http::LatencyUnit;
use tower_http::services::ServeDir;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{debug, info};

/// Runs the web server with both main and health check endpoints.
///
/// Starts two servers:
/// - Main server on the specified port (web UI + API)
/// - Health check server on port 8081
pub async fn run_server(pool: PgPool, port: u16) {
    let health_port = 8081;

    debug!(
        port = port,
        health_port = health_port,
        "Configuring web server"
    );

    // API routes
    let api_routes = Router::new()
        .route("/games", post(create_game_api))
        .route("/games/{game_id}/guess", post(make_guess_api))
        .with_state(pool.clone());

    // Web UI routes
    let web_routes = Router::new()
        .route("/game/new", post(create_game_web))
        .route("/game/{game_id}/guess", post(make_guess_web))
        .route("/difficulty-preview", get(difficulty_preview))
        .with_state(pool.clone());

    // Main application routes with tracing middleware
    let app = Router::new()
        .nest("/api", api_routes)
        .merge(web_routes)
        .fallback_service(ServeDir::new("static"))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(tracing::Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        );

    // Health check server (separate port)
    let health_app = Router::new()
        .route("/health", get(health_check))
        .with_state(pool.clone());

    let main_addr = format!("0.0.0.0:{}", port);
    let health_addr = format!("0.0.0.0:{}", health_port);

    let main_listener = tokio::net::TcpListener::bind(&main_addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {}", main_addr));

    let health_listener = tokio::net::TcpListener::bind(&health_addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {}", health_addr));

    // Log server startup info to stderr (structured logs)
    info!(
        main_addr = %main_addr,
        health_addr = %health_addr,
        main_port = port,
        health_port = health_port,
        "Starting web server"
    );
    info!(url = %format!("http://{}/", main_addr), "Web Interface available");
    info!("API Endpoints available");
    debug!("  POST /api/games - Create a new game");
    debug!("  POST /api/games/:game_id/guess - Make a guess");
    info!(url = %format!("http://{}/health", health_addr), "Health check available");

    // Create channels to signal when each server task has started
    let (main_ready_tx, main_ready_rx) = tokio::sync::oneshot::channel();
    let (health_ready_tx, health_ready_rx) = tokio::sync::oneshot::channel();

    // Spawn main server task
    let main_server = tokio::spawn(async move {
        debug!("Main server task started, beginning to accept connections");
        // Signal that we're about to start serving (listener is already bound and ready)
        let _ = main_ready_tx.send(());

        axum::serve(main_listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .unwrap_or_else(|_| panic!("Main server failed"));
    });

    // Spawn health check server task
    let health_server = tokio::spawn(async move {
        debug!("Health check server task started, beginning to accept connections");
        // Signal that we're about to start serving (listener is already bound and ready)
        let _ = health_ready_tx.send(());

        axum::serve(health_listener, health_app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .unwrap_or_else(|_| panic!("Health check server failed"));
    });

    // Wait for both servers to signal they're ready
    debug!("Waiting for server tasks to start accepting connections...");
    let _ = main_ready_rx.await;
    let _ = health_ready_rx.await;
    debug!("Both server tasks are now accepting connections");

    // Emit ready marker to stdout for tests/orchestration tools
    // (stdout is for program output, stderr is for logs)
    // This is only emitted AFTER both server tasks have started their accept loops
    println!("READY");

    // Wait for both servers to complete
    let _ = tokio::join!(main_server, health_server);
}

/// Graceful shutdown signal handler.
///
/// Listens for SIGINT (Ctrl+C) and initiates graceful shutdown of both servers.
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for shutdown signal");
    info!("Shutdown signal received, starting graceful shutdown");
}
