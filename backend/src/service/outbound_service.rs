use crate::dto::outbound::{
    CompleteOutboundRequest, CreateOutboundOrderRequest, OutboundOrderDetailResponse,
    OutboundOrderListQuery, OutboundOrderListResponse, UpdateOutboundOrderRequest,
};
use crate::error::AppError;
use crate::model::outbound::OutboundOrderStatus;
use crate::repository::inventory_repo::InventoryRepository;
use crate::repository::location_repo::LocationRepository;
use crate::repository::outbound_repo::OutboundOrderRepository;
use crate::repository::product_repo::ProductRepository;
use crate::repository::warehouse_repo::WarehouseRepository;
use crate::service::inventory_service::{InventoryService, StockDelta};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

const REF_OUTBOUND_COMPLETE: &str = "outbound_order_complete";
const REF_OUTBOUND_CANCEL: &str = "outbound_order_cancel";

pub struct OutboundService;

impl OutboundService {
    /* -------------------------------------------------------------- */
    /*  Create                                                          */
    /* -------------------------------------------------------------- */

    pub async fn create(
        pool: &PgPool,
        req: CreateOutboundOrderRequest,
    ) -> Result<OutboundOrderDetailResponse, AppError> {
        if req.items.is_empty() {
            return Err(AppError::Validation("至少需要一条明细".into()));
        }
        for it in &req.items {
            if it.planned_qty <= Decimal::ZERO {
                return Err(AppError::Validation("计划数量必须大于0".into()));
            }
        }

        WarehouseRepository::find_by_id(pool, req.warehouse_id)
            .await?
            .ok_or_else(|| AppError::NotFound("仓库不存在".into()))?;

        for it in &req.items {
            ProductRepository::find_by_id(pool, it.product_id)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("商品 {} 不存在", it.product_id)))?;

            let loc = LocationRepository::find_by_id(pool, it.location_id)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("库位 {} 不存在", it.location_id)))?;
            if loc.warehouse_id != req.warehouse_id {
                return Err(AppError::Validation("库位不属于目标仓库".into()));
            }
        }

        let order_no = Self::generate_order_no(pool).await?;
        let mut tx = pool.begin().await?;

        let order = OutboundOrderRepository::insert_order(
            &mut *tx,
            &order_no,
            req.warehouse_id,
            req.order_type,
            req.remark.as_deref(),
        )
        .await?;

        let tuples: Vec<(Uuid, Uuid, Decimal)> = req
            .items
            .iter()
            .map(|i| (i.product_id, i.location_id, i.planned_qty))
            .collect();
        OutboundOrderRepository::insert_items(&mut *tx, order.id, &tuples).await?;

        tx.commit().await?;

        OutboundOrderRepository::find_by_id_with_items(pool, order.id)
            .await?
            .ok_or_else(|| AppError::Internal("订单创建后读取失败".into()))
    }

    /* -------------------------------------------------------------- */
    /*  Update                                                          */
    /* -------------------------------------------------------------- */

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: UpdateOutboundOrderRequest,
    ) -> Result<OutboundOrderDetailResponse, AppError> {
        if req.items.is_empty() {
            return Err(AppError::Validation("至少需要一条明细".into()));
        }
        for it in &req.items {
            if it.planned_qty <= Decimal::ZERO {
                return Err(AppError::Validation("计划数量必须大于0".into()));
            }
        }

        let mut tx = pool.begin().await?;

        let order = OutboundOrderRepository::find_by_id_exec(&mut *tx, id)
            .await?
            .ok_or_else(|| AppError::NotFound("出库单不存在".into()))?;

        if order.status != OutboundOrderStatus::Draft {
            return Err(AppError::BusinessRule("仅草稿状态可编辑".into()));
        }

        WarehouseRepository::find_by_id(pool, req.warehouse_id)
            .await?
            .ok_or_else(|| AppError::NotFound("仓库不存在".into()))?;

        for it in &req.items {
            ProductRepository::find_by_id(pool, it.product_id)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("商品 {} 不存在", it.product_id)))?;

            let loc = LocationRepository::find_by_id(pool, it.location_id)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("库位 {} 不存在", it.location_id)))?;
            if loc.warehouse_id != req.warehouse_id {
                return Err(AppError::Validation("库位不属于目标仓库".into()));
            }
        }

        OutboundOrderRepository::update_order(
            &mut *tx,
            id,
            req.warehouse_id,
            req.order_type,
            req.remark.as_deref(),
        )
        .await?;
        OutboundOrderRepository::delete_items(&mut *tx, id).await?;

        let tuples: Vec<(Uuid, Uuid, Decimal)> = req
            .items
            .iter()
            .map(|i| (i.product_id, i.location_id, i.planned_qty))
            .collect();
        OutboundOrderRepository::insert_items(&mut *tx, id, &tuples).await?;

        // CAS no-op — verify still Draft
        let verified = OutboundOrderRepository::cas_status(
            &mut *tx,
            id,
            OutboundOrderStatus::Draft,
            OutboundOrderStatus::Draft,
            None,
        )
        .await?;
        if verified.is_none() {
            return Err(AppError::BusinessRule("订单状态已变更，无法编辑".into()));
        }

        tx.commit().await?;

        OutboundOrderRepository::find_by_id_with_items(pool, id)
            .await?
            .ok_or_else(|| AppError::Internal("订单更新后读取失败".into()))
    }

    /* -------------------------------------------------------------- */
    /*  Complete — decrease stock                                      */
    /* -------------------------------------------------------------- */

    pub async fn complete(
        pool: &PgPool,
        id: Uuid,
        req: CompleteOutboundRequest,
    ) -> Result<OutboundOrderDetailResponse, AppError> {
        if req.items.is_empty() {
            return Err(AppError::Validation("至少需要一条明细".into()));
        }
        {
            let mut seen = std::collections::HashSet::new();
            for ri in &req.items {
                if ri.actual_qty < Decimal::ZERO {
                    return Err(AppError::Validation("实发数量不能为负数".into()));
                }
                if !seen.insert(ri.item_id) {
                    return Err(AppError::Validation(format!("重复的明细 {}", ri.item_id)));
                }
            }
        }

        let mut tx = pool.begin().await?;

        let order = OutboundOrderRepository::find_by_id_exec(&mut *tx, id)
            .await?
            .ok_or_else(|| AppError::NotFound("出库单不存在".into()))?;

        if order.status != OutboundOrderStatus::Draft {
            return Err(AppError::BusinessRule("仅草稿状态可完成".into()));
        }

        let items = OutboundOrderRepository::find_items_exec(&mut *tx, id).await?;

        if req.items.len() != items.len() {
            return Err(AppError::Validation("必须填写所有明细的实发数量".into()));
        }
        let item_ids: std::collections::HashSet<Uuid> = items.iter().map(|it| it.id).collect();
        for ri in &req.items {
            if !item_ids.contains(&ri.item_id) {
                return Err(AppError::NotFound(format!(
                    "明细 {} 不属于本订单",
                    ri.item_id
                )));
            }
        }

        let item_map: std::collections::HashMap<Uuid, (Uuid, Uuid)> = items
            .iter()
            .map(|it| (it.id, (it.product_id, it.location_id)))
            .collect();

        // Write actual quantities
        for ri in &req.items {
            OutboundOrderRepository::update_item_actual(&mut *tx, ri.item_id, ri.actual_qty)
                .await?;
        }

        let deltas: Vec<StockDelta> = req
            .items
            .iter()
            .map(|ri| {
                let (product_id, location_id) = item_map[&ri.item_id];
                StockDelta {
                    product_id,
                    warehouse_id: order.warehouse_id,
                    location_id,
                    quantity: ri.actual_qty,
                }
            })
            .collect();

        // Pre-check stock levels inside the tx — produce a detailed
        // shortage list before attempting the decrease.
        let mut shortages: Vec<String> = Vec::new();
        for d in &deltas {
            let available = InventoryRepository::find_by_keys_exec(
                &mut *tx,
                d.product_id,
                d.warehouse_id,
                d.location_id,
            )
            .await?
            .map(|inv| inv.quantity)
            .unwrap_or(Decimal::ZERO);

            if available < d.quantity {
                shortages.push(format!(
                    "product={} location={} need={} avail={}",
                    d.product_id, d.location_id, d.quantity, available
                ));
            }
        }

        if !shortages.is_empty() {
            return Err(AppError::BusinessRule(format!(
                "库存不足: {}",
                shortages.join("; ")
            )));
        }

        InventoryService::decrease_stock_in_tx(&mut *tx, &deltas, REF_OUTBOUND_COMPLETE, order.id)
            .await?;

        let updated = OutboundOrderRepository::cas_status(
            &mut *tx,
            id,
            OutboundOrderStatus::Draft,
            OutboundOrderStatus::Completed,
            Some(chrono::Utc::now()),
        )
        .await?;
        if updated.is_none() {
            return Err(AppError::BusinessRule("订单状态已变更，无法完成".into()));
        }

        tx.commit().await?;

        OutboundOrderRepository::find_by_id_with_items(pool, id)
            .await?
            .ok_or_else(|| AppError::Internal("订单完成后读取失败".into()))
    }

    /* -------------------------------------------------------------- */
    /*  Cancel                                                          */
    /* -------------------------------------------------------------- */

    pub async fn cancel(pool: &PgPool, id: Uuid) -> Result<OutboundOrderDetailResponse, AppError> {
        let mut tx = pool.begin().await?;

        let order = OutboundOrderRepository::find_by_id_exec(&mut *tx, id)
            .await?
            .ok_or_else(|| AppError::NotFound("出库单不存在".into()))?;

        match order.status {
            OutboundOrderStatus::Draft => {
                let updated = OutboundOrderRepository::cas_status(
                    &mut *tx,
                    id,
                    OutboundOrderStatus::Draft,
                    OutboundOrderStatus::Cancelled,
                    None,
                )
                .await?;
                if updated.is_none() {
                    return Err(AppError::BusinessRule("订单状态已变更，无法取消".into()));
                }
            }
            OutboundOrderStatus::Completed => {
                let items = OutboundOrderRepository::find_items_exec(&mut *tx, id).await?;

                let deltas: Vec<StockDelta> = items
                    .iter()
                    .filter_map(|it| {
                        it.actual_qty.map(|qty| StockDelta {
                            product_id: it.product_id,
                            warehouse_id: order.warehouse_id,
                            location_id: it.location_id,
                            quantity: qty,
                        })
                    })
                    .collect();

                // Rollback: increase stock (reverse the decrease from complete)
                InventoryService::increase_stock_in_tx(
                    &mut *tx,
                    &deltas,
                    REF_OUTBOUND_CANCEL,
                    order.id,
                )
                .await?;

                let updated = OutboundOrderRepository::cas_status(
                    &mut *tx,
                    id,
                    OutboundOrderStatus::Completed,
                    OutboundOrderStatus::Cancelled,
                    None,
                )
                .await?;
                if updated.is_none() {
                    return Err(AppError::BusinessRule("订单状态已变更，无法取消".into()));
                }
            }
            OutboundOrderStatus::Cancelled => {
                return Err(AppError::BusinessRule("订单已取消".into()));
            }
        }

        tx.commit().await?;

        OutboundOrderRepository::find_by_id_with_items(pool, id)
            .await?
            .ok_or_else(|| AppError::Internal("订单取消后读取失败".into()))
    }

    /* -------------------------------------------------------------- */
    /*  List / Get by id                                                */
    /* -------------------------------------------------------------- */

    pub async fn get_by_id(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<OutboundOrderDetailResponse, AppError> {
        OutboundOrderRepository::find_by_id_with_items(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("出库单不存在".into()))
    }

    pub async fn list(
        pool: &PgPool,
        query: OutboundOrderListQuery,
    ) -> Result<OutboundOrderListResponse, AppError> {
        let (items, total) = OutboundOrderRepository::list(
            pool,
            query.keyword.as_deref(),
            query.warehouse_id,
            query.status,
            query.start_date.as_deref(),
            query.end_date.as_deref(),
            query.page,
            query.page_size,
        )
        .await?;

        Ok(OutboundOrderListResponse {
            items,
            total,
            page: query.page,
            page_size: query.page_size,
        })
    }

    /* -------------------------------------------------------------- */
    /*  helpers                                                         */
    /* -------------------------------------------------------------- */

    async fn generate_order_no(pool: &PgPool) -> Result<String, AppError> {
        let today = chrono::Utc::now().format("%Y%m%d").to_string();
        let prefix = format!("OUT{}", today);

        let row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM outbound_orders WHERE order_no LIKE $1")
                .bind(format!("{}%", &prefix))
                .fetch_one(pool)
                .await?;

        Ok(format!("{}{:06}", prefix, row.0 + 1))
    }
}
