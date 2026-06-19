use crate::dto::inbound::{
    CompleteInboundRequest, CreateInboundOrderRequest, InboundOrderDetailResponse,
    InboundOrderListQuery, InboundOrderListResponse, UpdateInboundOrderRequest,
};
use crate::error::AppError;
use crate::model::inbound::InboundOrderStatus;
use crate::repository::inbound_repo::InboundOrderRepository;
use crate::repository::location_repo::LocationRepository;
use crate::repository::product_repo::ProductRepository;
use crate::repository::warehouse_repo::WarehouseRepository;
use crate::service::inventory_service::{InventoryService, StockDelta};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

/// 库存流水 `reference_type`：区分"完成入库"与"取消已完成入库"。
const REF_INBOUND_COMPLETE: &str = "inbound_order_complete";
const REF_INBOUND_CANCEL: &str = "inbound_order_cancel";

pub struct InboundService;

impl InboundService {
    /* -------------------------------------------------------------- */
    /*  Create                                                          */
    /* -------------------------------------------------------------- */

    pub async fn create(
        pool: &PgPool,
        req: CreateInboundOrderRequest,
    ) -> Result<InboundOrderDetailResponse, AppError> {
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

        let order = InboundOrderRepository::insert_order(
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
        InboundOrderRepository::insert_items(&mut *tx, order.id, &tuples).await?;

        tx.commit().await?;

        InboundOrderRepository::find_by_id_with_items(pool, order.id)
            .await?
            .ok_or_else(|| AppError::Internal("订单创建后读取失败".into()))
    }

    /* -------------------------------------------------------------- */
    /*  Update — all inside one transaction                            */
    /* -------------------------------------------------------------- */

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: UpdateInboundOrderRequest,
    ) -> Result<InboundOrderDetailResponse, AppError> {
        if req.items.is_empty() {
            return Err(AppError::Validation("至少需要一条明细".into()));
        }
        for it in &req.items {
            if it.planned_qty <= Decimal::ZERO {
                return Err(AppError::Validation("计划数量必须大于0".into()));
            }
        }

        let mut tx = pool.begin().await?;

        // Read + validate inside tx
        let order = InboundOrderRepository::find_by_id_exec(&mut *tx, id)
            .await?
            .ok_or_else(|| AppError::NotFound("入库单不存在".into()))?;

        if order.status != InboundOrderStatus::Draft {
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

        InboundOrderRepository::update_order(
            &mut *tx,
            id,
            req.warehouse_id,
            req.order_type,
            req.remark.as_deref(),
        )
        .await?;
        InboundOrderRepository::delete_items(&mut *tx, id).await?;

        let tuples: Vec<(Uuid, Uuid, Decimal)> = req
            .items
            .iter()
            .map(|i| (i.product_id, i.location_id, i.planned_qty))
            .collect();
        InboundOrderRepository::insert_items(&mut *tx, id, &tuples).await?;

        // 【并发安全】CAS 空操作：确认写入完成后订单仍为 Draft。
        // 若并发 complete/cancel 已修改状态，cas_status 返回 None → 回滚。
        let verified = InboundOrderRepository::cas_status(
            &mut *tx,
            id,
            InboundOrderStatus::Draft,
            InboundOrderStatus::Draft,
            None,
        )
        .await?;

        if verified.is_none() {
            return Err(AppError::BusinessRule("订单状态已变更，无法编辑".into()));
        }

        tx.commit().await?;

        InboundOrderRepository::find_by_id_with_items(pool, id)
            .await?
            .ok_or_else(|| AppError::Internal("订单更新后读取失败".into()))
    }

    /* -------------------------------------------------------------- */
    /*  Complete — CAS from draft, all in one tx                       */
    /* -------------------------------------------------------------- */

    pub async fn complete(
        pool: &PgPool,
        id: Uuid,
        req: CompleteInboundRequest,
    ) -> Result<InboundOrderDetailResponse, AppError> {
        if req.items.is_empty() {
            return Err(AppError::Validation("至少需要一条明细".into()));
        }
        {
            let mut seen = std::collections::HashSet::new();
            for ri in &req.items {
                if ri.actual_qty < Decimal::ZERO {
                    return Err(AppError::Validation("实收数量不能为负数".into()));
                }
                if !seen.insert(ri.item_id) {
                    return Err(AppError::Validation(format!("重复的明细 {}", ri.item_id)));
                }
            }
        }

        let mut tx = pool.begin().await?;

        // Read order inside tx
        let order = InboundOrderRepository::find_by_id_exec(&mut *tx, id)
            .await?
            .ok_or_else(|| AppError::NotFound("入库单不存在".into()))?;

        // Read items inside tx
        let items = InboundOrderRepository::find_items_exec(&mut *tx, id).await?;

        // All order items must be covered
        if req.items.len() != items.len() {
            return Err(AppError::Validation("必须填写所有明细的实收数量".into()));
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

        // Write actual_qty
        for ri in &req.items {
            InboundOrderRepository::update_item_actual(&mut *tx, ri.item_id, ri.actual_qty).await?;
        }

        // Increase stock
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

        InventoryService::increase_stock_in_tx(&mut *tx, &deltas, REF_INBOUND_COMPLETE, order.id)
            .await?;

        // 【并发安全】CAS: draft → completed。若并发请求已完成/取消此订单，
        // cas_status 返回 None → 事务整体回滚（库存增加+流水都会撤销）。
        let updated = InboundOrderRepository::cas_status(
            &mut *tx,
            id,
            InboundOrderStatus::Draft,
            InboundOrderStatus::Completed,
            Some(chrono::Utc::now()),
        )
        .await?;

        if updated.is_none() {
            return Err(AppError::BusinessRule("订单状态已变更，无法完成".into()));
        }

        tx.commit().await?;

        InboundOrderRepository::find_by_id_with_items(pool, id)
            .await?
            .ok_or_else(|| AppError::Internal("订单完成后读取失败".into()))
    }

    /* -------------------------------------------------------------- */
    /*  Cancel — CAS from current status                               */
    /* -------------------------------------------------------------- */

    pub async fn cancel(pool: &PgPool, id: Uuid) -> Result<InboundOrderDetailResponse, AppError> {
        let mut tx = pool.begin().await?;

        // Read order inside tx
        let order = InboundOrderRepository::find_by_id_exec(&mut *tx, id)
            .await?
            .ok_or_else(|| AppError::NotFound("入库单不存在".into()))?;

        match order.status {
            InboundOrderStatus::Draft => {
                let updated = InboundOrderRepository::cas_status(
                    &mut *tx,
                    id,
                    InboundOrderStatus::Draft,
                    InboundOrderStatus::Cancelled,
                    None,
                )
                .await?;

                if updated.is_none() {
                    return Err(AppError::BusinessRule("订单状态已变更，无法取消".into()));
                }
            }
            InboundOrderStatus::Completed => {
                // Read items inside tx
                let items = InboundOrderRepository::find_items_exec(&mut *tx, id).await?;

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

                // Rollback stock first
                InventoryService::decrease_stock_in_tx(
                    &mut *tx,
                    &deltas,
                    REF_INBOUND_CANCEL,
                    order.id,
                )
                .await?;

                // CAS: completed → cancelled
                let updated = InboundOrderRepository::cas_status(
                    &mut *tx,
                    id,
                    InboundOrderStatus::Completed,
                    InboundOrderStatus::Cancelled,
                    None,
                )
                .await?;

                if updated.is_none() {
                    return Err(AppError::BusinessRule("订单状态已变更，无法取消".into()));
                }
            }
            InboundOrderStatus::Cancelled => {
                return Err(AppError::BusinessRule("订单已取消".into()));
            }
        }

        tx.commit().await?;

        InboundOrderRepository::find_by_id_with_items(pool, id)
            .await?
            .ok_or_else(|| AppError::Internal("订单取消后读取失败".into()))
    }

    /* -------------------------------------------------------------- */
    /*  List                                                            */
    /* -------------------------------------------------------------- */

    pub async fn get_by_id(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<InboundOrderDetailResponse, AppError> {
        InboundOrderRepository::find_by_id_with_items(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("入库单不存在".into()))
    }

    pub async fn list(
        pool: &PgPool,
        query: InboundOrderListQuery,
    ) -> Result<InboundOrderListResponse, AppError> {
        let (items, total) = InboundOrderRepository::list(
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

        Ok(InboundOrderListResponse {
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
        let prefix = format!("IN{}", today);

        let row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM inbound_orders WHERE order_no LIKE $1")
                .bind(format!("{}%", &prefix))
                .fetch_one(pool)
                .await?;

        Ok(format!("{}{:06}", prefix, row.0 + 1))
    }
}
