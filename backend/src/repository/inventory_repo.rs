use crate::model::inventory::Inventory;
use rust_decimal::Decimal;
use sqlx::{Executor, PgPool, Postgres};
use uuid::Uuid;

pub struct InventoryRepository;

impl InventoryRepository {
    /// Upsert: insert a new inventory row, or add to the existing quantity.
    pub async fn upsert<'e, E: Executor<'e, Database = Postgres>>(
        executor: E,
        product_id: Uuid,
        warehouse_id: Uuid,
        location_id: Uuid,
        delta: Decimal,
    ) -> Result<Inventory, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO inventories (product_id, warehouse_id, location_id, quantity)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (product_id, warehouse_id, location_id)
            DO UPDATE SET quantity    = inventories.quantity + $4,
                          updated_at = now()
            RETURNING *
            "#,
        )
        .bind(product_id)
        .bind(warehouse_id)
        .bind(location_id)
        .bind(delta)
        .fetch_one(executor)
        .await
    }

    pub async fn find_by_keys(
        pool: &PgPool,
        product_id: Uuid,
        warehouse_id: Uuid,
        location_id: Uuid,
    ) -> Result<Option<Inventory>, sqlx::Error> {
        Self::find_by_keys_exec(pool, product_id, warehouse_id, location_id).await
    }

    /// Same as `find_by_keys` but works inside a transaction.
    pub async fn find_by_keys_exec<'e, E: Executor<'e, Database = Postgres>>(
        executor: E,
        product_id: Uuid,
        warehouse_id: Uuid,
        location_id: Uuid,
    ) -> Result<Option<Inventory>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT id, product_id, warehouse_id, location_id,
                   quantity, created_at, updated_at
            FROM inventories
            WHERE product_id   = $1
              AND warehouse_id = $2
              AND location_id  = $3
            "#,
        )
        .bind(product_id)
        .bind(warehouse_id)
        .bind(location_id)
        .fetch_optional(executor)
        .await
    }

    /// Decrease inventory quantity (optimistic concurrency check).
    pub async fn decrease<'e, E: Executor<'e, Database = Postgres>>(
        executor: E,
        product_id: Uuid,
        warehouse_id: Uuid,
        location_id: Uuid,
        delta: Decimal,
    ) -> Result<Option<Inventory>, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE inventories
            SET quantity    = quantity - $4,
                updated_at  = now()
            WHERE product_id   = $1
              AND warehouse_id = $2
              AND location_id  = $3
              AND quantity     >= $4
            RETURNING *
            "#,
        )
        .bind(product_id)
        .bind(warehouse_id)
        .bind(location_id)
        .bind(delta)
        .fetch_optional(executor)
        .await
    }
}
