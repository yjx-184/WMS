use crate::model::inventory::TransactionChangeType;
use crate::repository::inventory_repo::InventoryRepository;
use crate::repository::transaction_repo::TransactionRepository;
use rust_decimal::Decimal;
use sqlx::{Executor, PgPool, Postgres};
use uuid::Uuid;

pub struct InventoryService;

/// 库存变更描述：在 `(product_id, warehouse_id, location_id)` 三元组上
/// 增加或减少 `quantity`。
pub struct StockDelta {
    pub product_id: Uuid,
    pub warehouse_id: Uuid,
    pub location_id: Uuid,
    pub quantity: Decimal,
}

impl InventoryService {
    /* -------------------------------------------------------------- */
    /*  Standalone (own transaction)                                    */
    /* -------------------------------------------------------------- */

    pub async fn increase_stock(
        pool: &PgPool,
        deltas: &[StockDelta],
        reference_type: &str,
        reference_id: Uuid,
    ) -> Result<(), crate::error::AppError> {
        let mut tx = pool.begin().await?;
        Self::increase_stock_in_tx(&mut *tx, deltas, reference_type, reference_id).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn decrease_stock(
        pool: &PgPool,
        deltas: &[StockDelta],
        reference_type: &str,
        reference_id: Uuid,
    ) -> Result<(), crate::error::AppError> {
        let mut tx = pool.begin().await?;
        Self::decrease_stock_in_tx(&mut *tx, deltas, reference_type, reference_id).await?;
        tx.commit().await?;
        Ok(())
    }

    /* -------------------------------------------------------------- */
    /*  In-transaction — quantity_before derived from the operation     */
    /* -------------------------------------------------------------- */

    /// 在已有事务中增加库存，并写入 `change_type=inbound` 流水。
    ///
    /// `quantity_before` = `after.quantity - delta`，从 upsert 原子操作
    /// 的返回值反推，避免先读后写导致的并发窗口。
    pub async fn increase_stock_in_tx<'e, E>(
        tx: &mut E,
        deltas: &[StockDelta],
        reference_type: &str,
        reference_id: Uuid,
    ) -> Result<(), crate::error::AppError>
    where
        for<'a> &'a mut E: Executor<'a, Database = Postgres>,
    {
        for d in deltas {
            let after = InventoryRepository::upsert(
                &mut *tx,
                d.product_id,
                d.warehouse_id,
                d.location_id,
                d.quantity,
            )
            .await?;

            let before = after.quantity - d.quantity;

            TransactionRepository::insert(
                &mut *tx,
                d.product_id,
                d.warehouse_id,
                d.location_id,
                TransactionChangeType::Inbound,
                d.quantity,
                before,
                after.quantity,
                reference_type,
                reference_id,
            )
            .await?;
        }
        Ok(())
    }

    /// 在已有事务中扣减库存，并写入 `change_type=outbound` 流水。
    ///
    /// 若库存不足（`decrease` 返回 None），事务回滚并返回 `BusinessRule` 错误。
    /// `quantity_before` = `after.quantity + delta`。
    pub async fn decrease_stock_in_tx<'e, E>(
        tx: &mut E,
        deltas: &[StockDelta],
        reference_type: &str,
        reference_id: Uuid,
    ) -> Result<(), crate::error::AppError>
    where
        for<'a> &'a mut E: Executor<'a, Database = Postgres>,
    {
        for d in deltas {
            let after = InventoryRepository::decrease(
                &mut *tx,
                d.product_id,
                d.warehouse_id,
                d.location_id,
                d.quantity,
            )
            .await?
            .ok_or_else(|| crate::error::AppError::BusinessRule("库存不足".into()))?;

            let before = after.quantity + d.quantity;

            TransactionRepository::insert(
                &mut *tx,
                d.product_id,
                d.warehouse_id,
                d.location_id,
                TransactionChangeType::Outbound,
                d.quantity,
                before,
                after.quantity,
                reference_type,
                reference_id,
            )
            .await?;
        }
        Ok(())
    }
}
