use crate::model::warehouse::{Warehouse, WarehouseStatus};
use sqlx::PgPool;
use uuid::Uuid;

/// Data-access layer for the `warehouses` table.
///
/// All SQL is parameterised; business rules belong in the service layer.
pub struct WarehouseRepository;

impl WarehouseRepository {
    /* -------------------------------------------------------------- */
    /*  Queries                                                        */
    /* -------------------------------------------------------------- */

    /// Paginated list with optional keyword and status filters.
    ///
    /// Keyword searches `code` and `name` case-insensitively.
    pub async fn list(
        pool: &PgPool,
        keyword: Option<&str>,
        status: Option<WarehouseStatus>,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<Warehouse>, i64), sqlx::Error> {
        let keyword_param = keyword.map(|k| format!("%{}%", k));
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let limit = page_size as i64;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) AS count
            FROM warehouses
            WHERE ($1::text IS NULL OR code ILIKE $1 OR name ILIKE $1)
              AND ($2::warehouse_status IS NULL OR status = $2)
            "#,
        )
        .bind(&keyword_param)
        .bind(&status)
        .fetch_one(pool)
        .await?;

        let items: Vec<Warehouse> = sqlx::query_as(
            r#"
            SELECT id, code, name, address, status, created_at, updated_at
            FROM warehouses
            WHERE ($1::text IS NULL OR code ILIKE $1 OR name ILIKE $1)
              AND ($2::warehouse_status IS NULL OR status = $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(&keyword_param)
        .bind(&status)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok((items, total.0))
    }

    /// Look up a single warehouse by primary key.
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Warehouse>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT id, code, name, address, status, created_at, updated_at
            FROM warehouses
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// Look up a warehouse by code (unique constraint).
    pub async fn find_by_code(pool: &PgPool, code: &str) -> Result<Option<Warehouse>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT id, code, name, address, status, created_at, updated_at
            FROM warehouses
            WHERE code = $1
            "#,
        )
        .bind(code)
        .fetch_optional(pool)
        .await
    }

    /* -------------------------------------------------------------- */
    /*  Mutations                                                      */
    /* -------------------------------------------------------------- */

    /// Insert a new warehouse. Returns the row with generated `id` and timestamps.
    pub async fn insert(
        pool: &PgPool,
        code: &str,
        name: &str,
        address: Option<&str>,
    ) -> Result<Warehouse, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO warehouses (code, name, address)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(code)
        .bind(name)
        .bind(address)
        .fetch_one(pool)
        .await
    }

    /// Update warehouse fields.
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        code: &str,
        name: &str,
        address: Option<&str>,
    ) -> Result<Warehouse, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE warehouses
            SET code       = $2,
                name       = $3,
                address    = $4,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(code)
        .bind(name)
        .bind(address)
        .fetch_one(pool)
        .await
    }

    /// Toggle warehouse status (active ↔ disabled).
    pub async fn update_status(
        pool: &PgPool,
        id: Uuid,
        status: WarehouseStatus,
    ) -> Result<Warehouse, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE warehouses
            SET status     = $2,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&status)
        .fetch_one(pool)
        .await
    }
}
