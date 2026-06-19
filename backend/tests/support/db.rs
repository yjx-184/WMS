use backend::db;
use sqlx::PgPool;

/// Build a pool connected to the TEST_DATABASE_URL and run migrations.
/// Loads `.env.test` if present.  No global cleanup is performed;
/// each test manages its own data via unique identifiers.
pub async fn setup_test_db() -> PgPool {
    let _ = dotenvy::from_filename(concat!(env!("CARGO_MANIFEST_DIR"), "/.env.test"));

    let url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must be set in .env.test or environment");

    let pool = db::create_pool(&backend::config::Config {
        database_url: url,
        server_port: 0,
    })
    .await;

    // Safety check — panic if not a test database
    let row: (String,) = sqlx::query_as("SELECT current_database()")
        .fetch_one(&pool)
        .await
        .expect("Failed to read current database name");

    assert!(
        row.0 == "wms_test" || row.0.ends_with("_test"),
        "Refusing to use non-test database '{}'. \
         TEST_DATABASE_URL must point to a database named 'wms_test' or ending with '_test'.",
        row.0
    );

    pool
}

/// Clean up fixtures created by a test, identified by their unique codes.
/// FK-safe deletion order.
pub async fn cleanup_fixtures(
    pool: &PgPool,
    product_sku: &str,
    warehouse_code: &str,
    location_code: &str,
) {
    // Find the IDs first
    let product: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM products WHERE sku_code = $1")
            .bind(product_sku)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
    let warehouse: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM warehouses WHERE code = $1")
            .bind(warehouse_code)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
    let location: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM locations WHERE code = $1")
            .bind(location_code)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    if let Some((loc_id,)) = location {
        let _ = sqlx::query("DELETE FROM inventory_transactions WHERE location_id = $1")
            .bind(loc_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM inventories WHERE location_id = $1")
            .bind(loc_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM locations WHERE id = $1")
            .bind(loc_id)
            .execute(pool)
            .await;
    }
    if let Some((wh_id,)) = warehouse {
        let _ = sqlx::query("DELETE FROM warehouses WHERE id = $1")
            .bind(wh_id)
            .execute(pool)
            .await;
    }
    if let Some((prod_id,)) = product {
        let _ = sqlx::query("DELETE FROM products WHERE id = $1")
            .bind(prod_id)
            .execute(pool)
            .await;
    }
}
