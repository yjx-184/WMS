use crate::config::Config;
use sqlx::postgres::{PgPool, PgPoolOptions};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

/// Create a PostgreSQL connection pool and run pending migrations.
///
/// # Panics
///
/// Panics if the database is unreachable or migrations fail.
pub async fn create_pool(config: &Config) -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    pool
}
