mod support;

use backend::service::inventory_service::{InventoryService, StockDelta};
use rust_decimal::Decimal;
use sqlx::PgPool;
use support::db;
use uuid::Uuid;

struct Fixtures {
    pool: PgPool,
    product_id: Uuid,
    warehouse_id: Uuid,
    location_id: Uuid,
    product_sku: String,
    warehouse_code: String,
    location_code: String,
}

impl Fixtures {
    async fn setup(suffix: &str) -> Self {
        let pool = db::setup_test_db().await;
        let product_sku = format!("TST-{}-{}", suffix, Uuid::new_v4());
        let warehouse_code = format!("TWH-{}", suffix);
        let location_code = format!("LOC-{}", suffix);

        let product: (Uuid,) = sqlx::query_as(
            "INSERT INTO products (sku_code, name, unit) VALUES ($1, 'Test', 'pcs') RETURNING id",
        )
        .bind(&product_sku)
        .fetch_one(&pool)
        .await
        .unwrap();

        let wh: (Uuid,) = sqlx::query_as(
            "INSERT INTO warehouses (code, name) VALUES ($1, 'TestWH') RETURNING id",
        )
        .bind(&warehouse_code)
        .fetch_one(&pool)
        .await
        .unwrap();

        let loc: (Uuid,) = sqlx::query_as(
            "INSERT INTO locations (warehouse_id, code) VALUES ($1, $2) RETURNING id",
        )
        .bind(wh.0)
        .bind(&location_code)
        .fetch_one(&pool)
        .await
        .unwrap();

        Fixtures {
            pool,
            product_id: product.0,
            warehouse_id: wh.0,
            location_id: loc.0,
            product_sku,
            warehouse_code,
            location_code,
        }
    }

    fn delta(&self, qty: i64) -> StockDelta {
        StockDelta {
            product_id: self.product_id,
            warehouse_id: self.warehouse_id,
            location_id: self.location_id,
            quantity: Decimal::new(qty, 0),
        }
    }

    async fn cleanup(&self) {
        db::cleanup_fixtures(
            &self.pool,
            &self.product_sku,
            &self.warehouse_code,
            &self.location_code,
        )
        .await;
    }
}

#[tokio::test]
async fn increase_stock_new() {
    let f = Fixtures::setup("inc1").await;
    let ref_id = Uuid::new_v4();

    InventoryService::increase_stock(&f.pool, &[f.delta(100)], "test", ref_id)
        .await
        .unwrap();

    let qty: (Decimal,) = sqlx::query_as(
        "SELECT quantity FROM inventories WHERE product_id=$1 AND warehouse_id=$2 AND location_id=$3",
    )
    .bind(f.product_id).bind(f.warehouse_id).bind(f.location_id)
    .fetch_one(&f.pool).await.unwrap();
    assert_eq!(qty.0, Decimal::new(100, 0));

    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM inventory_transactions WHERE reference_id=$1")
            .bind(ref_id)
            .fetch_one(&f.pool)
            .await
            .unwrap();
    assert_eq!(count.0, 1);

    f.cleanup().await;
}

#[tokio::test]
async fn increase_stock_accumulate() {
    let f = Fixtures::setup("inc2").await;

    InventoryService::increase_stock(&f.pool, &[f.delta(50)], "test", Uuid::new_v4())
        .await
        .unwrap();
    InventoryService::increase_stock(&f.pool, &[f.delta(50)], "test", Uuid::new_v4())
        .await
        .unwrap();

    let qty: (Decimal,) = sqlx::query_as(
        "SELECT quantity FROM inventories WHERE product_id=$1 AND warehouse_id=$2 AND location_id=$3",
    )
    .bind(f.product_id).bind(f.warehouse_id).bind(f.location_id)
    .fetch_one(&f.pool).await.unwrap();
    assert_eq!(qty.0, Decimal::new(100, 0));

    f.cleanup().await;
}

#[tokio::test]
async fn decrease_stock_normal() {
    let f = Fixtures::setup("dec1").await;
    let ref_id = Uuid::new_v4();

    InventoryService::increase_stock(&f.pool, &[f.delta(100)], "test", Uuid::new_v4())
        .await
        .unwrap();
    InventoryService::decrease_stock(&f.pool, &[f.delta(30)], "test", ref_id)
        .await
        .unwrap();

    let qty: (Decimal,) = sqlx::query_as(
        "SELECT quantity FROM inventories WHERE product_id=$1 AND warehouse_id=$2 AND location_id=$3",
    )
    .bind(f.product_id).bind(f.warehouse_id).bind(f.location_id)
    .fetch_one(&f.pool).await.unwrap();
    assert_eq!(qty.0, Decimal::new(70, 0));

    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM inventory_transactions WHERE reference_id=$1 AND change_type='outbound'",
    )
    .bind(ref_id).fetch_one(&f.pool).await.unwrap();
    assert_eq!(count.0, 1);

    f.cleanup().await;
}

#[tokio::test]
async fn decrease_stock_to_zero() {
    let f = Fixtures::setup("dec2").await;

    InventoryService::increase_stock(&f.pool, &[f.delta(10)], "test", Uuid::new_v4())
        .await
        .unwrap();
    InventoryService::decrease_stock(&f.pool, &[f.delta(10)], "test", Uuid::new_v4())
        .await
        .unwrap();

    let qty: (Decimal,) = sqlx::query_as(
        "SELECT quantity FROM inventories WHERE product_id=$1 AND warehouse_id=$2 AND location_id=$3",
    )
    .bind(f.product_id).bind(f.warehouse_id).bind(f.location_id)
    .fetch_one(&f.pool).await.unwrap();
    assert_eq!(qty.0, Decimal::new(0, 0));

    f.cleanup().await;
}

#[tokio::test]
async fn decrease_stock_insufficient() {
    let f = Fixtures::setup("dec3").await;

    InventoryService::increase_stock(&f.pool, &[f.delta(5)], "test", Uuid::new_v4())
        .await
        .unwrap();
    let result =
        InventoryService::decrease_stock(&f.pool, &[f.delta(100)], "test", Uuid::new_v4()).await;

    assert!(result.is_err());

    let qty: (Decimal,) = sqlx::query_as(
        "SELECT quantity FROM inventories WHERE product_id=$1 AND warehouse_id=$2 AND location_id=$3",
    )
    .bind(f.product_id).bind(f.warehouse_id).bind(f.location_id)
    .fetch_one(&f.pool).await.unwrap();
    assert_eq!(qty.0, Decimal::new(5, 0));

    f.cleanup().await;
}
