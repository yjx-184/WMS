use crate::dto::outbound::{
    OutboundOrderDetailItem, OutboundOrderDetailResponse, OutboundOrderListItem,
};
use crate::model::outbound::{
    OutboundOrder, OutboundOrderItem, OutboundOrderStatus, OutboundOrderType,
};
use rust_decimal::Decimal;
use sqlx::{Executor, PgPool, Postgres};
use uuid::Uuid;

pub struct OutboundOrderRepository;

impl OutboundOrderRepository {
    /* -------------------------------------------------------------- */
    /*  List                                                            */
    /* -------------------------------------------------------------- */

    pub async fn list(
        pool: &PgPool,
        keyword: Option<&str>,
        warehouse_id: Option<Uuid>,
        status: Option<OutboundOrderStatus>,
        start_date: Option<&str>,
        end_date: Option<&str>,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<OutboundOrderListItem>, i64), sqlx::Error> {
        let keyword_param = keyword.map(|k| format!("%{}%", k));
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let limit = page_size as i64;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) AS count
            FROM outbound_orders o
            WHERE ($1::text IS NULL OR o.order_no ILIKE $1)
              AND ($2::uuid IS NULL OR o.warehouse_id = $2)
              AND ($3::outbound_order_status IS NULL OR o.status = $3)
              AND ($4::timestamptz IS NULL OR o.created_at >= $4::timestamptz)
              AND ($5::timestamptz IS NULL OR o.created_at <  $5::timestamptz + INTERVAL '1 day')
            "#,
        )
        .bind(&keyword_param)
        .bind(&warehouse_id)
        .bind(&status)
        .bind(start_date.map(|d| format!("{}T00:00:00Z", d)))
        .bind(end_date.map(|d| format!("{}T00:00:00Z", d)))
        .fetch_one(pool)
        .await?;

        let items: Vec<OutboundOrderListItem> = sqlx::query_as(
            r#"
            SELECT o.id, o.order_no, o.warehouse_id,
                   COALESCE(w.name, '') AS warehouse_name,
                   o.order_type, o.status, o.remark,
                   o.completed_at, o.created_at, o.updated_at
            FROM outbound_orders o
            LEFT JOIN warehouses w ON w.id = o.warehouse_id
            WHERE ($1::text IS NULL OR o.order_no ILIKE $1)
              AND ($2::uuid IS NULL OR o.warehouse_id = $2)
              AND ($3::outbound_order_status IS NULL OR o.status = $3)
              AND ($4::timestamptz IS NULL OR o.created_at >= $4::timestamptz)
              AND ($5::timestamptz IS NULL OR o.created_at <  $5::timestamptz + INTERVAL '1 day')
            ORDER BY o.created_at DESC
            LIMIT $6 OFFSET $7
            "#,
        )
        .bind(&keyword_param)
        .bind(&warehouse_id)
        .bind(&status)
        .bind(start_date.map(|d| format!("{}T00:00:00Z", d)))
        .bind(end_date.map(|d| format!("{}T00:00:00Z", d)))
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok((items, total.0))
    }

    /* -------------------------------------------------------------- */
    /*  Detail                                                          */
    /* -------------------------------------------------------------- */

    pub async fn find_by_id_with_items(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<Option<OutboundOrderDetailResponse>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct DetailHeader {
            id: Uuid,
            order_no: String,
            warehouse_id: Uuid,
            warehouse_name: String,
            order_type: OutboundOrderType,
            status: OutboundOrderStatus,
            remark: Option<String>,
            completed_at: Option<chrono::DateTime<chrono::Utc>>,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let header: Option<DetailHeader> = sqlx::query_as(
            r#"
            SELECT o.id, o.order_no, o.warehouse_id,
                   COALESCE(w.name, '') AS warehouse_name,
                   o.order_type, o.status, o.remark,
                   o.completed_at, o.created_at, o.updated_at
            FROM outbound_orders o
            LEFT JOIN warehouses w ON w.id = o.warehouse_id
            WHERE o.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        match header {
            None => Ok(None),
            Some(h) => {
                let items: Vec<OutboundOrderDetailItem> = sqlx::query_as(
                    r#"
                    SELECT i.id, i.product_id,
                           COALESCE(p.name, '')     AS product_name,
                           COALESCE(p.sku_code, '') AS sku_code,
                           i.location_id,
                           COALESCE(l.code, '')     AS location_code,
                           i.planned_qty, i.actual_qty,
                           i.created_at
                    FROM outbound_order_items i
                    LEFT JOIN products  p ON p.id = i.product_id
                    LEFT JOIN locations l ON l.id = i.location_id
                    WHERE i.order_id = $1
                    ORDER BY i.created_at
                    "#,
                )
                .bind(id)
                .fetch_all(pool)
                .await?;

                Ok(Some(OutboundOrderDetailResponse {
                    id: h.id,
                    order_no: h.order_no,
                    warehouse_id: h.warehouse_id,
                    warehouse_name: h.warehouse_name,
                    order_type: h.order_type,
                    status: h.status,
                    remark: h.remark,
                    completed_at: h.completed_at,
                    created_at: h.created_at,
                    updated_at: h.updated_at,
                    items,
                }))
            }
        }
    }

    /* -------------------------------------------------------------- */
    /*  Order CRUD                                                      */
    /* -------------------------------------------------------------- */

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<OutboundOrder>, sqlx::Error> {
        Self::find_by_id_exec(pool, id).await
    }

    pub async fn find_by_id_exec<'e, E: Executor<'e, Database = Postgres>>(
        executor: E,
        id: Uuid,
    ) -> Result<Option<OutboundOrder>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT id, order_no, warehouse_id, order_type, status,
                   remark, completed_at, created_at, updated_at
            FROM outbound_orders WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(executor)
        .await
    }

    pub async fn insert_order<'e, E: Executor<'e, Database = Postgres>>(
        executor: E,
        order_no: &str,
        warehouse_id: Uuid,
        order_type: OutboundOrderType,
        remark: Option<&str>,
    ) -> Result<OutboundOrder, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO outbound_orders (order_no, warehouse_id, order_type, remark)
            VALUES ($1, $2, $3, $4) RETURNING *
            "#,
        )
        .bind(order_no)
        .bind(warehouse_id)
        .bind(&order_type)
        .bind(remark)
        .fetch_one(executor)
        .await
    }

    pub async fn update_order<'e, E: Executor<'e, Database = Postgres>>(
        executor: E,
        id: Uuid,
        warehouse_id: Uuid,
        order_type: OutboundOrderType,
        remark: Option<&str>,
    ) -> Result<OutboundOrder, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE outbound_orders
            SET warehouse_id = $2, order_type = $3, remark = $4, updated_at = now()
            WHERE id = $1 RETURNING *
            "#,
        )
        .bind(id)
        .bind(warehouse_id)
        .bind(&order_type)
        .bind(remark)
        .fetch_one(executor)
        .await
    }

    pub async fn update_status<'e, E: Executor<'e, Database = Postgres>>(
        executor: E,
        id: Uuid,
        status: OutboundOrderStatus,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<OutboundOrder, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE outbound_orders
            SET status = $2, completed_at = $3, updated_at = now()
            WHERE id = $1 RETURNING *
            "#,
        )
        .bind(id)
        .bind(&status)
        .bind(completed_at)
        .fetch_one(executor)
        .await
    }

    pub async fn cas_status<'e, E: Executor<'e, Database = Postgres>>(
        executor: E,
        id: Uuid,
        expected: OutboundOrderStatus,
        new_status: OutboundOrderStatus,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Option<OutboundOrder>, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE outbound_orders
            SET status       = $3,
                completed_at = COALESCE($4, completed_at),
                updated_at   = now()
            WHERE id      = $1
              AND status  = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&expected)
        .bind(&new_status)
        .bind(completed_at)
        .fetch_optional(executor)
        .await
    }

    /* -------------------------------------------------------------- */
    /*  Items                                                           */
    /* -------------------------------------------------------------- */

    pub async fn find_items(
        pool: &PgPool,
        order_id: Uuid,
    ) -> Result<Vec<OutboundOrderItem>, sqlx::Error> {
        Self::find_items_exec(pool, order_id).await
    }

    pub async fn find_items_exec<'e, E: Executor<'e, Database = Postgres>>(
        executor: E,
        order_id: Uuid,
    ) -> Result<Vec<OutboundOrderItem>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT id, order_id, product_id, location_id,
                   planned_qty, actual_qty, created_at
            FROM outbound_order_items
            WHERE order_id = $1
            ORDER BY created_at
            "#,
        )
        .bind(order_id)
        .fetch_all(executor)
        .await
    }

    pub async fn insert_items<'e, E>(
        executor: &mut E,
        order_id: Uuid,
        items: &[(Uuid, Uuid, Decimal)],
    ) -> Result<Vec<OutboundOrderItem>, sqlx::Error>
    where
        for<'a> &'a mut E: Executor<'a, Database = Postgres>,
    {
        let mut result = Vec::with_capacity(items.len());
        for (product_id, location_id, planned_qty) in items {
            let item = sqlx::query_as(
                r#"
                INSERT INTO outbound_order_items (order_id, product_id, location_id, planned_qty)
                VALUES ($1, $2, $3, $4) RETURNING *
                "#,
            )
            .bind(order_id)
            .bind(product_id)
            .bind(location_id)
            .bind(planned_qty)
            .fetch_one(&mut *executor)
            .await?;
            result.push(item);
        }
        Ok(result)
    }

    pub async fn delete_items<'e, E: Executor<'e, Database = Postgres>>(
        executor: E,
        order_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM outbound_order_items WHERE order_id = $1")
            .bind(order_id)
            .execute(executor)
            .await?;
        Ok(())
    }

    pub async fn update_item_actual<'e, E: Executor<'e, Database = Postgres>>(
        executor: E,
        item_id: Uuid,
        actual_qty: Decimal,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE outbound_order_items SET actual_qty = $2 WHERE id = $1")
            .bind(item_id)
            .bind(actual_qty)
            .execute(executor)
            .await?;
        Ok(())
    }
}
