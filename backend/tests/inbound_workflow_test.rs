mod support;

use backend::dto::inbound::{
    CompleteInboundItemRequest, CompleteInboundRequest, CreateInboundItemRequest,
    CreateInboundOrderRequest,
};
use backend::model::inbound::InboundOrderStatus;
use backend::service::inbound_service::InboundService;
use rust_decimal::Decimal;
use sqlx::PgPool;
use support::db;
use uuid::Uuid;

struct TestContext {
    pool: PgPool,
    product_skus: Vec<String>,
    warehouse_code: String,
    location_codes: Vec<String>,
}

impl TestContext {
    async fn setup(suffix: &str) -> Self {
        let pool = db::setup_test_db().await;

        let wh_code = format!("IBW-{}", suffix);
        let loc1 = format!("IBL1-{}", suffix);
        let loc2 = format!("IBL2-{}", suffix);
        let sku1 = format!("IBP1-{}", suffix);
        let sku2 = format!("IBP2-{}", suffix);

        // Warehouse
        sqlx::query("INSERT INTO warehouses (code, name) VALUES ($1, 'TestWH')")
            .bind(&wh_code)
            .execute(&pool)
            .await
            .unwrap();

        // Products
        for sku in &[&sku1, &sku2] {
            sqlx::query("INSERT INTO products (sku_code, name, unit) VALUES ($1, 'Test', 'pcs')")
                .bind(sku)
                .execute(&pool)
                .await
                .unwrap();
        }

        // Locations
        let wh_id: (Uuid,) = sqlx::query_as("SELECT id FROM warehouses WHERE code = $1")
            .bind(&wh_code)
            .fetch_one(&pool)
            .await
            .unwrap();
        for loc in &[&loc1, &loc2] {
            sqlx::query("INSERT INTO locations (warehouse_id, code) VALUES ($1, $2)")
                .bind(wh_id.0)
                .bind(loc)
                .execute(&pool)
                .await
                .unwrap();
        }

        TestContext {
            pool,
            product_skus: vec![sku1, sku2],
            warehouse_code: wh_code,
            location_codes: vec![loc1, loc2],
        }
    }

    async fn product_id(&self, idx: usize) -> Uuid {
        let row: (Uuid,) = sqlx::query_as("SELECT id FROM products WHERE sku_code = $1")
            .bind(&self.product_skus[idx])
            .fetch_one(&self.pool)
            .await
            .unwrap();
        row.0
    }

    async fn warehouse_id(&self) -> Uuid {
        let row: (Uuid,) = sqlx::query_as("SELECT id FROM warehouses WHERE code = $1")
            .bind(&self.warehouse_code)
            .fetch_one(&self.pool)
            .await
            .unwrap();
        row.0
    }

    async fn location_id(&self, idx: usize) -> Uuid {
        let row: (Uuid,) = sqlx::query_as("SELECT id FROM locations WHERE code = $1")
            .bind(&self.location_codes[idx])
            .fetch_one(&self.pool)
            .await
            .unwrap();
        row.0
    }

    async fn inventory_quantity(
        &self,
        product_id: Uuid,
        location_id: Uuid,
        warehouse_id: Uuid,
    ) -> Decimal {
        let row: Option<(Decimal,)> = sqlx::query_as(
            "SELECT quantity FROM inventories WHERE product_id=$1 AND warehouse_id=$2 AND location_id=$3",
        )
        .bind(product_id)
        .bind(warehouse_id)
        .bind(location_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap();
        row.map(|r| r.0).unwrap_or(Decimal::ZERO)
    }

    async fn cleanup(&self) {
        for loc in &self.location_codes {
            let loc_id: Option<(Uuid,)> =
                sqlx::query_as("SELECT id FROM locations WHERE code = $1")
                    .bind(loc)
                    .fetch_optional(&self.pool)
                    .await
                    .unwrap();
            if let Some((id,)) = loc_id {
                let _ = sqlx::query("DELETE FROM inventory_transactions WHERE location_id = $1")
                    .bind(id)
                    .execute(&self.pool)
                    .await;
                let _ = sqlx::query("DELETE FROM inventories WHERE location_id = $1")
                    .bind(id)
                    .execute(&self.pool)
                    .await;
                let _ = sqlx::query("DELETE FROM inbound_order_items WHERE location_id = $1")
                    .bind(id)
                    .execute(&self.pool)
                    .await;
                let _ = sqlx::query("DELETE FROM locations WHERE id = $1")
                    .bind(id)
                    .execute(&self.pool)
                    .await;
            }
        }
        let wh_id: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM warehouses WHERE code = $1")
            .bind(&self.warehouse_code)
            .fetch_optional(&self.pool)
            .await
            .unwrap();
        if let Some((id,)) = wh_id {
            let _ = sqlx::query("DELETE FROM inbound_orders WHERE warehouse_id = $1")
                .bind(id)
                .execute(&self.pool)
                .await;
            let _ = sqlx::query("DELETE FROM warehouses WHERE id = $1")
                .bind(id)
                .execute(&self.pool)
                .await;
        }
        for sku in &self.product_skus {
            let _ = sqlx::query("DELETE FROM products WHERE sku_code = $1")
                .bind(sku)
                .execute(&self.pool)
                .await;
        }
    }
}

#[tokio::test]
async fn inbound_create_complete_cancel_workflow() {
    let ctx = TestContext::setup("wf1").await;
    let pid1 = ctx.product_id(0).await;
    let pid2 = ctx.product_id(1).await;
    let wid = ctx.warehouse_id().await;
    let lid1 = ctx.location_id(0).await;
    let lid2 = ctx.location_id(1).await;

    // ── 1. Create inbound order (2 items) ────────────────────────
    let order = InboundService::create(
        &ctx.pool,
        CreateInboundOrderRequest {
            warehouse_id: wid,
            order_type: Default::default(),
            remark: None,
            items: vec![
                CreateInboundItemRequest {
                    product_id: pid1,
                    location_id: lid1,
                    planned_qty: Decimal::new(100, 0),
                },
                CreateInboundItemRequest {
                    product_id: pid2,
                    location_id: lid2,
                    planned_qty: Decimal::new(200, 0),
                },
            ],
        },
    )
    .await
    .unwrap();

    assert_eq!(order.status, InboundOrderStatus::Draft);
    assert_eq!(order.items.len(), 2);

    // ── 2. Complete ──────────────────────────────────────────────
    let item_ids: Vec<Uuid> = order.items.iter().map(|it| it.id).collect();

    InboundService::complete(
        &ctx.pool,
        order.id,
        CompleteInboundRequest {
            items: vec![
                CompleteInboundItemRequest {
                    item_id: item_ids[0],
                    actual_qty: Decimal::new(90, 0),
                },
                CompleteInboundItemRequest {
                    item_id: item_ids[1],
                    actual_qty: Decimal::new(180, 0),
                },
            ],
        },
    )
    .await
    .unwrap();

    // Verify inventory increased
    assert_eq!(
        ctx.inventory_quantity(pid1, lid1, wid).await,
        Decimal::new(90, 0)
    );
    assert_eq!(
        ctx.inventory_quantity(pid2, lid2, wid).await,
        Decimal::new(180, 0)
    );

    // Verify inbound transaction records (quantity / before / after)
    #[derive(sqlx::FromRow)]
    struct TxRow {
        quantity: Decimal,
        quantity_before: Decimal,
        quantity_after: Decimal,
    }

    // Item 1: 0 → 90
    let tx1: TxRow = sqlx::query_as(
        "SELECT quantity, quantity_before, quantity_after
         FROM inventory_transactions
         WHERE reference_id=$1 AND change_type='inbound'
           AND product_id=$2 AND location_id=$3",
    )
    .bind(order.id)
    .bind(pid1)
    .bind(lid1)
    .fetch_one(&ctx.pool)
    .await
    .unwrap();
    assert_eq!(tx1.quantity, Decimal::new(90, 0));
    assert_eq!(tx1.quantity_before, Decimal::ZERO);
    assert_eq!(tx1.quantity_after, Decimal::new(90, 0));

    // Item 2: 0 → 180
    let tx2: TxRow = sqlx::query_as(
        "SELECT quantity, quantity_before, quantity_after
         FROM inventory_transactions
         WHERE reference_id=$1 AND change_type='inbound'
           AND product_id=$2 AND location_id=$3",
    )
    .bind(order.id)
    .bind(pid2)
    .bind(lid2)
    .fetch_one(&ctx.pool)
    .await
    .unwrap();
    assert_eq!(tx2.quantity, Decimal::new(180, 0));
    assert_eq!(tx2.quantity_before, Decimal::ZERO);
    assert_eq!(tx2.quantity_after, Decimal::new(180, 0));

    // Verify status
    let status_row: (InboundOrderStatus,) =
        sqlx::query_as("SELECT status FROM inbound_orders WHERE id = $1")
            .bind(order.id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    assert_eq!(status_row.0, InboundOrderStatus::Completed);

    // ── 3. Cancel completed order (rollback stock) ─────────────
    InboundService::cancel(&ctx.pool, order.id).await.unwrap();

    // Verify inventory rolled back
    assert_eq!(ctx.inventory_quantity(pid1, lid1, wid).await, Decimal::ZERO);
    assert_eq!(ctx.inventory_quantity(pid2, lid2, wid).await, Decimal::ZERO);

    // Verify outbound rollback transaction records
    // Item 1: 90 → 0
    let otx1: TxRow = sqlx::query_as(
        "SELECT quantity, quantity_before, quantity_after
         FROM inventory_transactions
         WHERE reference_id=$1 AND change_type='outbound'
           AND product_id=$2 AND location_id=$3",
    )
    .bind(order.id)
    .bind(pid1)
    .bind(lid1)
    .fetch_one(&ctx.pool)
    .await
    .unwrap();
    assert_eq!(otx1.quantity, Decimal::new(90, 0));
    assert_eq!(otx1.quantity_before, Decimal::new(90, 0));
    assert_eq!(otx1.quantity_after, Decimal::ZERO);

    // Item 2: 180 → 0
    let otx2: TxRow = sqlx::query_as(
        "SELECT quantity, quantity_before, quantity_after
         FROM inventory_transactions
         WHERE reference_id=$1 AND change_type='outbound'
           AND product_id=$2 AND location_id=$3",
    )
    .bind(order.id)
    .bind(pid2)
    .bind(lid2)
    .fetch_one(&ctx.pool)
    .await
    .unwrap();
    assert_eq!(otx2.quantity, Decimal::new(180, 0));
    assert_eq!(otx2.quantity_before, Decimal::new(180, 0));
    assert_eq!(otx2.quantity_after, Decimal::ZERO);

    // Verify status
    let status_row: (InboundOrderStatus,) =
        sqlx::query_as("SELECT status FROM inbound_orders WHERE id = $1")
            .bind(order.id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    assert_eq!(status_row.0, InboundOrderStatus::Cancelled);

    ctx.cleanup().await;
}
