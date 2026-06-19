use axum::Router;
use axum::middleware::from_fn;
use backend::db::AppState;
use backend::middleware;
use sqlx::PgPool;

/// Build the Axum app for integration testing.
pub fn test_app(pool: PgPool) -> Router {
    let state = AppState { pool };
    backend::router::create_router(state)
        .layer(from_fn(middleware::request_id))
        .layer(middleware::cors_layer())
}
