use crate::dto::inventory::{
    InventoryListResponse, InventoryQueryParams, InventoryRow, TransactionListResponse,
    TransactionQueryParams, TransactionRow,
};
use sqlx::PgPool;

pub struct StockQueryService;

impl StockQueryService {
    /* -------------------------------------------------------------- */
    /*  Inventory query                                                 */
    /* -------------------------------------------------------------- */

    pub async fn query_inventory(
        pool: &PgPool,
        params: InventoryQueryParams,
    ) -> Result<InventoryListResponse, sqlx::Error> {
        let keyword_param = params.keyword.as_deref().map(|k| format!("%{}%", k));
        let offset = ((params.page.saturating_sub(1)) * params.page_size) as i64;
        let limit = params.page_size as i64;

        // Shared WHERE clause (used by both COUNT and items)
        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) AS count
            FROM inventories     inv
            JOIN products   p ON p.id = inv.product_id
            JOIN warehouses w ON w.id = inv.warehouse_id
            JOIN locations  l ON l.id = inv.location_id
            WHERE ($1::uuid IS NULL OR inv.product_id   = $1)
              AND ($2::uuid IS NULL OR inv.warehouse_id = $2)
              AND ($3::uuid IS NULL OR inv.location_id  = $3)
              AND ($4::text IS NULL OR p.name    ILIKE $4
                                    OR p.sku_code ILIKE $4
                                    OR w.name    ILIKE $4
                                    OR l.code    ILIKE $4)
            "#,
        )
        .bind(&params.product_id)
        .bind(&params.warehouse_id)
        .bind(&params.location_id)
        .bind(&keyword_param)
        .fetch_one(pool)
        .await?;

        let items: Vec<InventoryRow> = sqlx::query_as(
            r#"
            SELECT inv.id, inv.product_id,
                   COALESCE(p.name, '')      AS product_name,
                   COALESCE(p.sku_code, '')  AS sku_code,
                   inv.warehouse_id,
                   COALESCE(w.name, '')      AS warehouse_name,
                   inv.location_id,
                   COALESCE(l.code, '')      AS location_code,
                   inv.quantity, inv.created_at, inv.updated_at
            FROM inventories     inv
            JOIN products   p ON p.id = inv.product_id
            JOIN warehouses w ON w.id = inv.warehouse_id
            JOIN locations  l ON l.id = inv.location_id
            WHERE ($1::uuid IS NULL OR inv.product_id   = $1)
              AND ($2::uuid IS NULL OR inv.warehouse_id = $2)
              AND ($3::uuid IS NULL OR inv.location_id  = $3)
              AND ($4::text IS NULL OR p.name    ILIKE $4
                                    OR p.sku_code ILIKE $4
                                    OR w.name    ILIKE $4
                                    OR l.code    ILIKE $4)
            ORDER BY inv.updated_at DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(&params.product_id)
        .bind(&params.warehouse_id)
        .bind(&params.location_id)
        .bind(&keyword_param)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(InventoryListResponse {
            items,
            total: total.0,
            page: params.page,
            page_size: params.page_size,
        })
    }

    /* -------------------------------------------------------------- */
    /*  Transaction query                                               */
    /* -------------------------------------------------------------- */

    pub async fn query_transactions(
        pool: &PgPool,
        params: TransactionQueryParams,
    ) -> Result<TransactionListResponse, sqlx::Error> {
        let offset = ((params.page.saturating_sub(1)) * params.page_size) as i64;
        let limit = params.page_size as i64;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) AS count
            FROM inventory_transactions t
            JOIN products   p ON p.id = t.product_id
            JOIN warehouses w ON w.id = t.warehouse_id
            JOIN locations  l ON l.id = t.location_id
            WHERE ($1::uuid IS NULL OR t.product_id   = $1)
              AND ($2::uuid IS NULL OR t.warehouse_id = $2)
              AND ($3::uuid IS NULL OR t.location_id  = $3)
              AND ($4::text IS NULL OR t.change_type::text = $4)
              AND ($5::timestamptz IS NULL OR t.created_at >= $5::timestamptz)
              AND ($6::timestamptz IS NULL OR t.created_at <  $6::timestamptz + INTERVAL '1 day')
            "#,
        )
        .bind(&params.product_id)
        .bind(&params.warehouse_id)
        .bind(&params.location_id)
        .bind(&params.change_type)
        .bind(
            params
                .start_date
                .as_deref()
                .map(|d| format!("{}T00:00:00Z", d)),
        )
        .bind(
            params
                .end_date
                .as_deref()
                .map(|d| format!("{}T00:00:00Z", d)),
        )
        .fetch_one(pool)
        .await?;

        let items: Vec<TransactionRow> = sqlx::query_as(
            r#"
            SELECT t.id, t.product_id,
                   COALESCE(p.name, '')     AS product_name,
                   COALESCE(p.sku_code, '') AS sku_code,
                   t.warehouse_id,
                   COALESCE(w.name, '')     AS warehouse_name,
                   t.location_id,
                   COALESCE(l.code, '')     AS location_code,
                   t.change_type::text      AS change_type,
                   t.quantity, t.quantity_before, t.quantity_after,
                   t.reference_type, t.reference_id, t.created_at
            FROM inventory_transactions t
            JOIN products   p ON p.id = t.product_id
            JOIN warehouses w ON w.id = t.warehouse_id
            JOIN locations  l ON l.id = t.location_id
            WHERE ($1::uuid IS NULL OR t.product_id   = $1)
              AND ($2::uuid IS NULL OR t.warehouse_id = $2)
              AND ($3::uuid IS NULL OR t.location_id  = $3)
              AND ($4::text IS NULL OR t.change_type::text = $4)
              AND ($5::timestamptz IS NULL OR t.created_at >= $5::timestamptz)
              AND ($6::timestamptz IS NULL OR t.created_at <  $6::timestamptz + INTERVAL '1 day')
            ORDER BY t.created_at DESC
            LIMIT $7 OFFSET $8
            "#,
        )
        .bind(&params.product_id)
        .bind(&params.warehouse_id)
        .bind(&params.location_id)
        .bind(&params.change_type)
        .bind(
            params
                .start_date
                .as_deref()
                .map(|d| format!("{}T00:00:00Z", d)),
        )
        .bind(
            params
                .end_date
                .as_deref()
                .map(|d| format!("{}T00:00:00Z", d)),
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(TransactionListResponse {
            items,
            total: total.0,
            page: params.page,
            page_size: params.page_size,
        })
    }
}
