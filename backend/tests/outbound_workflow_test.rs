mod support;

use backend::dto::inbound::{
    CompleteInboundItemRequest, CompleteInboundRequest, CreateInboundItemRequest,
    CreateInboundOrderRequest,
};
use backend::dto::outbound::{
    CompleteOutboundItemRequest, CompleteOutboundRequest, CreateOutboundItemRequest,
    CreateOutboundOrderRequest,
};
use backend::model::outbound::OutboundOrderStatus;
use backend::service::inbound_service::InboundService;
use backend::service::outbound_service::OutboundService;
use rust_decimal::Decimal;
use sqlx::PgPool;
use support::db;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct TxRow {
    quantity: Decimal,
    quantity_before: Decimal,
    quantity_after: Decimal,
}

struct TestContext {
    pool: PgPool,
    product_skus: Vec<String>,
    warehouse_code: String,
    location_codes: Vec<String>,
}

impl TestContext {
    async fn setup() -> Self {
        let pool = db::setup_test_db().await;
        // Unique suffix per run — prevents cross-run PK collisions
        let u = Uuid::new_v4()
            .to_string()
            .split('-')
            .next()
            .unwrap()
            .to_string();
        let wh = format!("OBW-{}", u);
        let l1 = format!("OBL1-{}", u);
        let l2 = format!("OBL2-{}", u);
        let s1 = format!("OBP1-{}", u);
        let s2 = format!("OBP2-{}", u);

        sqlx::query("INSERT INTO warehouses (code, name) VALUES ($1, 'T')")
            .bind(&wh)
            .execute(&pool)
            .await
            .unwrap();
        for s in &[&s1, &s2] {
            sqlx::query("INSERT INTO products (sku_code, name, unit) VALUES ($1, 'T', 'pcs')")
                .bind(s)
                .execute(&pool)
                .await
                .unwrap();
        }
        let wh_id: (Uuid,) =
            sqlx::query_as::<_, (Uuid,)>("SELECT id FROM warehouses WHERE code=$1")
                .bind(&wh)
                .fetch_one(&pool)
                .await
                .unwrap();
        for l in &[&l1, &l2] {
            sqlx::query("INSERT INTO locations (warehouse_id, code) VALUES ($1, $2)")
                .bind(wh_id.0)
                .bind(l)
                .execute(&pool)
                .await
                .unwrap();
        }
        TestContext {
            pool,
            product_skus: vec![s1, s2],
            warehouse_code: wh,
            location_codes: vec![l1, l2],
        }
    }

    async fn pid(&self, i: usize) -> Uuid {
        sqlx::query_as::<_, (Uuid,)>("SELECT id FROM products WHERE sku_code=$1")
            .bind(&self.product_skus[i])
            .fetch_one(&self.pool)
            .await
            .unwrap()
            .0
    }
    async fn wid(&self) -> Uuid {
        sqlx::query_as::<_, (Uuid,)>("SELECT id FROM warehouses WHERE code=$1")
            .bind(&self.warehouse_code)
            .fetch_one(&self.pool)
            .await
            .unwrap()
            .0
    }
    async fn lid(&self, i: usize) -> Uuid {
        sqlx::query_as::<_, (Uuid,)>("SELECT id FROM locations WHERE code=$1")
            .bind(&self.location_codes[i])
            .fetch_one(&self.pool)
            .await
            .unwrap()
            .0
    }
    async fn qty(&self, pid: Uuid, lid: Uuid, wid: Uuid) -> Decimal {
        sqlx::query_as::<_,(Decimal,)>("SELECT quantity FROM inventories WHERE product_id=$1 AND warehouse_id=$2 AND location_id=$3")
            .bind(pid).bind(wid).bind(lid).fetch_optional(&self.pool).await
            .unwrap().map(|r| r.0).unwrap_or(Decimal::ZERO)
    }

    /// FK-safe cleanup covering both inbound and outbound data.
    async fn cleanup(&self) {
        let mut loc_ids = Vec::new();
        for c in &self.location_codes {
            if let Ok(Some(r)) =
                sqlx::query_as::<_, (Uuid,)>("SELECT id FROM locations WHERE code=$1")
                    .bind(c)
                    .fetch_optional(&self.pool)
                    .await
            {
                loc_ids.push(r.0);
            }
        }
        let wh_id: Option<Uuid> =
            sqlx::query_as::<_, (Uuid,)>("SELECT id FROM warehouses WHERE code=$1")
                .bind(&self.warehouse_code)
                .fetch_optional(&self.pool)
                .await
                .unwrap()
                .map(|r| r.0);
        let mut prod_ids = Vec::new();
        for s in &self.product_skus {
            if let Ok(Some(r)) =
                sqlx::query_as::<_, (Uuid,)>("SELECT id FROM products WHERE sku_code=$1")
                    .bind(s)
                    .fetch_optional(&self.pool)
                    .await
            {
                prod_ids.push(r.0);
            }
        }

        // Child tables first
        for &lid in &loc_ids {
            let _ = sqlx::query("DELETE FROM outbound_order_items WHERE location_id=$1")
                .bind(lid)
                .execute(&self.pool)
                .await;
            let _ = sqlx::query("DELETE FROM inbound_order_items WHERE location_id=$1")
                .bind(lid)
                .execute(&self.pool)
                .await;
        }
        if let Some(wid) = wh_id {
            let _ = sqlx::query("DELETE FROM outbound_order_items WHERE order_id IN (SELECT id FROM outbound_orders WHERE warehouse_id=$1)").bind(wid).execute(&self.pool).await;
            let _ = sqlx::query("DELETE FROM inbound_order_items WHERE order_id IN (SELECT id FROM inbound_orders WHERE warehouse_id=$1)").bind(wid).execute(&self.pool).await;
            let _ = sqlx::query("DELETE FROM outbound_orders WHERE warehouse_id=$1")
                .bind(wid)
                .execute(&self.pool)
                .await;
            let _ = sqlx::query("DELETE FROM inbound_orders WHERE warehouse_id=$1")
                .bind(wid)
                .execute(&self.pool)
                .await;
        }
        for &lid in &loc_ids {
            let _ = sqlx::query("DELETE FROM inventory_transactions WHERE location_id=$1")
                .bind(lid)
                .execute(&self.pool)
                .await;
            let _ = sqlx::query("DELETE FROM inventories WHERE location_id=$1")
                .bind(lid)
                .execute(&self.pool)
                .await;
            let _ = sqlx::query("DELETE FROM locations WHERE id=$1")
                .bind(lid)
                .execute(&self.pool)
                .await;
        }
        if let Some(wid) = wh_id {
            let _ = sqlx::query("DELETE FROM warehouses WHERE id=$1")
                .bind(wid)
                .execute(&self.pool)
                .await;
        }
        for &pid in &prod_ids {
            let _ = sqlx::query("DELETE FROM products WHERE id=$1")
                .bind(pid)
                .execute(&self.pool)
                .await;
        }
    }
}

#[tokio::test]
async fn outbound_create_complete_cancel_workflow() {
    let ctx = TestContext::setup().await;
    let p1 = ctx.pid(0).await;
    let p2 = ctx.pid(1).await;
    let w = ctx.wid().await;
    let l1 = ctx.lid(0).await;
    let l2 = ctx.lid(1).await;

    // ── 1. Seed stock via inbound ────────────────────────────────
    let inbound = InboundService::create(
        &ctx.pool,
        CreateInboundOrderRequest {
            warehouse_id: w,
            order_type: Default::default(),
            remark: None,
            items: vec![
                CreateInboundItemRequest {
                    product_id: p1,
                    location_id: l1,
                    planned_qty: Decimal::new(100, 0),
                },
                CreateInboundItemRequest {
                    product_id: p2,
                    location_id: l2,
                    planned_qty: Decimal::new(50, 0),
                },
            ],
        },
    )
    .await
    .unwrap();
    let iids: Vec<Uuid> = inbound.items.iter().map(|it| it.id).collect();
    InboundService::complete(
        &ctx.pool,
        inbound.id,
        CompleteInboundRequest {
            items: vec![
                CompleteInboundItemRequest {
                    item_id: iids[0],
                    actual_qty: Decimal::new(100, 0),
                },
                CompleteInboundItemRequest {
                    item_id: iids[1],
                    actual_qty: Decimal::new(50, 0),
                },
            ],
        },
    )
    .await
    .unwrap();

    // ── 2. Create outbound (within stock) ────────────────────────
    let out1 = OutboundService::create(
        &ctx.pool,
        CreateOutboundOrderRequest {
            warehouse_id: w,
            order_type: Default::default(),
            remark: None,
            items: vec![
                CreateOutboundItemRequest {
                    product_id: p1,
                    location_id: l1,
                    planned_qty: Decimal::new(30, 0),
                },
                CreateOutboundItemRequest {
                    product_id: p2,
                    location_id: l2,
                    planned_qty: Decimal::new(20, 0),
                },
            ],
        },
    )
    .await
    .unwrap();
    let oids: Vec<Uuid> = out1.items.iter().map(|it| it.id).collect();

    // ── 3. Complete outbound ─────────────────────────────────────
    OutboundService::complete(
        &ctx.pool,
        out1.id,
        CompleteOutboundRequest {
            items: vec![
                CompleteOutboundItemRequest {
                    item_id: oids[0],
                    actual_qty: Decimal::new(30, 0),
                },
                CompleteOutboundItemRequest {
                    item_id: oids[1],
                    actual_qty: Decimal::new(20, 0),
                },
            ],
        },
    )
    .await
    .unwrap();

    assert_eq!(ctx.qty(p1, l1, w).await, Decimal::new(70, 0));
    assert_eq!(ctx.qty(p2, l2, w).await, Decimal::new(30, 0));

    let tx1: TxRow = sqlx::query_as(
        "SELECT quantity, quantity_before, quantity_after FROM inventory_transactions
         WHERE reference_id=$1 AND change_type='outbound' AND product_id=$2 AND location_id=$3",
    )
    .bind(out1.id)
    .bind(p1)
    .bind(l1)
    .fetch_one(&ctx.pool)
    .await
    .unwrap();
    assert_eq!(tx1.quantity, Decimal::new(30, 0));
    assert_eq!(tx1.quantity_before, Decimal::new(100, 0));
    assert_eq!(tx1.quantity_after, Decimal::new(70, 0));

    let tx2: TxRow = sqlx::query_as(
        "SELECT quantity, quantity_before, quantity_after FROM inventory_transactions
         WHERE reference_id=$1 AND change_type='outbound' AND product_id=$2 AND location_id=$3",
    )
    .bind(out1.id)
    .bind(p2)
    .bind(l2)
    .fetch_one(&ctx.pool)
    .await
    .unwrap();
    assert_eq!(tx2.quantity, Decimal::new(20, 0));
    assert_eq!(tx2.quantity_before, Decimal::new(50, 0));
    assert_eq!(tx2.quantity_after, Decimal::new(30, 0));

    let s: (OutboundOrderStatus,) =
        sqlx::query_as("SELECT status FROM outbound_orders WHERE id=$1")
            .bind(out1.id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    assert_eq!(s.0, OutboundOrderStatus::Completed);

    // ── 4. Over-stock outbound → should fail ─────────────────────
    let out2 = OutboundService::create(
        &ctx.pool,
        CreateOutboundOrderRequest {
            warehouse_id: w,
            order_type: Default::default(),
            remark: None,
            items: vec![CreateOutboundItemRequest {
                product_id: p1,
                location_id: l1,
                planned_qty: Decimal::new(999, 0),
            }],
        },
    )
    .await
    .unwrap();
    let oid2: Vec<Uuid> = out2.items.iter().map(|it| it.id).collect();
    let result = OutboundService::complete(
        &ctx.pool,
        out2.id,
        CompleteOutboundRequest {
            items: vec![CompleteOutboundItemRequest {
                item_id: oid2[0],
                actual_qty: Decimal::new(999, 0),
            }],
        },
    )
    .await;
    assert!(result.is_err());
    assert_eq!(ctx.qty(p1, l1, w).await, Decimal::new(70, 0));

    // ── 5. Cancel completed outbound → rollback stock ────────────
    OutboundService::cancel(&ctx.pool, out1.id).await.unwrap();

    assert_eq!(ctx.qty(p1, l1, w).await, Decimal::new(100, 0));
    assert_eq!(ctx.qty(p2, l2, w).await, Decimal::new(50, 0));

    // Rollback tx for p1/l1
    let rt1: TxRow = sqlx::query_as(
        "SELECT quantity, quantity_before, quantity_after FROM inventory_transactions
         WHERE reference_id=$1 AND change_type='inbound' AND product_id=$2 AND location_id=$3",
    )
    .bind(out1.id)
    .bind(p1)
    .bind(l1)
    .fetch_one(&ctx.pool)
    .await
    .unwrap();
    assert_eq!(rt1.quantity, Decimal::new(30, 0));
    assert_eq!(rt1.quantity_before, Decimal::new(70, 0));
    assert_eq!(rt1.quantity_after, Decimal::new(100, 0));

    // Rollback tx for p2/l2
    let rt2: TxRow = sqlx::query_as(
        "SELECT quantity, quantity_before, quantity_after FROM inventory_transactions
         WHERE reference_id=$1 AND change_type='inbound' AND product_id=$2 AND location_id=$3",
    )
    .bind(out1.id)
    .bind(p2)
    .bind(l2)
    .fetch_one(&ctx.pool)
    .await
    .unwrap();
    assert_eq!(rt2.quantity, Decimal::new(20, 0));
    assert_eq!(rt2.quantity_before, Decimal::new(30, 0));
    assert_eq!(rt2.quantity_after, Decimal::new(50, 0));

    let s: (OutboundOrderStatus,) =
        sqlx::query_as("SELECT status FROM outbound_orders WHERE id=$1")
            .bind(out1.id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    assert_eq!(s.0, OutboundOrderStatus::Cancelled);

    ctx.cleanup().await;
}
