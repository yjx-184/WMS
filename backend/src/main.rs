mod config;
mod db;
mod dto;
mod error;
mod middleware;
mod model;
mod repository;
mod router;

use axum::middleware::from_fn;

#[tokio::main]
async fn main() {
    middleware::init_tracing();

    let config = config::Config::from_env();

    println!("Starting WMS backend...");
    println!("DATABASE_URL: {}", config.masked_database_url());
    println!("SERVER_PORT: {}", config.server_port);

    let pool = db::create_pool(&config).await;
    let state = db::AppState { pool };

    let app = router::create_router(state)
        .layer(from_fn(middleware::request_id))
        .layer(middleware::cors_layer());

    let addr = format!("0.0.0.0:{}", config.server_port);
    println!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind TCP listener");

    axum::serve(listener, app).await.expect("Server crashed");
}
