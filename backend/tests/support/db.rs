use backend::db;
use sqlx::PgPool;

/// Build a pool connected to the TEST_DATABASE_URL and run migrations.
/// Loads `.env.test` if present.
pub async fn setup_test_db() -> PgPool {
    // Load .env.test from the backend directory
    let _ = dotenvy::from_filename(concat!(env!("CARGO_MANIFEST_DIR"), "/.env.test"));

    let url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must be set in .env.test or environment");

    let pool = db::create_pool(&backend::config::Config {
        database_url: url,
        server_port: 0,
    })
    .await;

    // ── Safety check: only clean test databases ─────────────────
    let row: (String,) = sqlx::query_as("SELECT current_database()")
        .fetch_one(&pool)
        .await
        .expect("Failed to read current database name");

    let db_name = &row.0;
    assert!(
        db_name == "wms_test" || db_name.ends_with("_test"),
        "Refusing to clean non-test database '{}'. \
         TEST_DATABASE_URL must point to a database named 'wms_test' or ending with '_test'.",
        db_name
    );

    // Clean up in FK-safe order
    let tables = [
        "inventory_transactions",
        "outbound_order_items",
        "outbound_orders",
        "inbound_order_items",
        "inbound_orders",
        "inventories",
        "locations",
        "warehouses",
        "products",
    ];
    for t in &tables {
        let q = format!("DELETE FROM {}", t);
        let _ = sqlx::query(&q).execute(&pool).await;
    }

    pool
}
