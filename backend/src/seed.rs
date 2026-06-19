use sqlx::PgPool;

/// Run seed data if all three master-data tables are empty.
pub async fn seed_if_empty(pool: &PgPool) {
    let p: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM products")
        .fetch_one(pool)
        .await
        .unwrap();
    let w: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM warehouses")
        .fetch_one(pool)
        .await
        .unwrap();
    let l: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM locations")
        .fetch_one(pool)
        .await
        .unwrap();

    if p.0 == 0 && w.0 == 0 && l.0 == 0 {
        println!("Database is empty, running seed data...");
        seed(pool).await;
    } else {
        println!(
            "Database already has data (p={} w={} l={}), skipping seed.",
            p.0, w.0, l.0
        );
    }
}

/// Execute the seed SQL from `backend/migrations/seed.sql` inside a
/// single transaction. Uses `raw_sql` (simple query protocol) which
/// supports multiple statements natively.
pub async fn seed(pool: &PgPool) {
    let sql = include_str!("../migrations/seed.sql");

    let mut tx = pool
        .begin()
        .await
        .expect("Failed to begin seed transaction");

    sqlx::raw_sql(sql)
        .execute(&mut *tx)
        .await
        .expect("Failed to execute seed SQL");

    tx.commit()
        .await
        .expect("Failed to commit seed transaction");

    println!("Seed data applied.");
}
