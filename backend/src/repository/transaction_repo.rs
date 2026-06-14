use crate::model::inventory::{InventoryTransaction, TransactionChangeType};
use rust_decimal::Decimal;
use sqlx::{Executor, Postgres};
use uuid::Uuid;

pub struct TransactionRepository;

impl TransactionRepository {
    /// Insert a single inventory-transaction audit record.
    #[allow(clippy::too_many_arguments)]
    pub async fn insert<'e, E: Executor<'e, Database = Postgres>>(
        executor: E,
        product_id: Uuid,
        warehouse_id: Uuid,
        location_id: Uuid,
        change_type: TransactionChangeType,
        quantity: Decimal,
        quantity_before: Decimal,
        quantity_after: Decimal,
        reference_type: &str,
        reference_id: Uuid,
    ) -> Result<InventoryTransaction, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO inventory_transactions
                (product_id, warehouse_id, location_id, change_type,
                 quantity, quantity_before, quantity_after,
                 reference_type, reference_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(product_id)
        .bind(warehouse_id)
        .bind(location_id)
        .bind(&change_type)
        .bind(quantity)
        .bind(quantity_before)
        .bind(quantity_after)
        .bind(reference_type)
        .bind(reference_id)
        .fetch_one(executor)
        .await
    }
}
