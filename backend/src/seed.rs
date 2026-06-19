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

/// 在单个事务中执行 `backend/migrations/seed.sql`（编译时嵌入）。
///
/// `sqlx::raw_sql` 使用 PostgreSQL simple query 协议原生支持多语句，
/// 避免手动解析 SQL。事务保证所有插入要么全部成功要么全部回滚。
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
