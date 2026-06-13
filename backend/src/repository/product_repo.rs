use crate::model::product::{Product, ProductStatus};
use sqlx::PgPool;
use uuid::Uuid;

/// Data-access layer for the `products` table.
///
/// All SQL is parameterised; business rules belong in the service layer.
pub struct ProductRepository;

impl ProductRepository {
    /* -------------------------------------------------------------- */
    /*  Queries                                                        */
    /* -------------------------------------------------------------- */

    /// Paginated list with optional keyword and status filters.
    ///
    /// Keyword searches `sku_code` and `name` case-insensitively.
    pub async fn list(
        pool: &PgPool,
        keyword: Option<&str>,
        status: Option<ProductStatus>,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<Product>, i64), sqlx::Error> {
        let keyword_param = keyword.map(|k| format!("%{}%", k));
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let limit = page_size as i64;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) AS count
            FROM products
            WHERE ($1::text IS NULL OR sku_code ILIKE $1 OR name ILIKE $1)
              AND ($2::product_status IS NULL OR status = $2)
            "#,
        )
        .bind(&keyword_param)
        .bind(&status)
        .fetch_one(pool)
        .await?;

        let items: Vec<Product> = sqlx::query_as(
            r#"
            SELECT id, sku_code, name, unit, spec, barcode,
                   status, created_at, updated_at
            FROM products
            WHERE ($1::text IS NULL OR sku_code ILIKE $1 OR name ILIKE $1)
              AND ($2::product_status IS NULL OR status = $2)
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

    /// Look up a single product by primary key.
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Product>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT id, sku_code, name, unit, spec, barcode,
                   status, created_at, updated_at
            FROM products
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// Look up a product by SKU code (unique constraint).
    pub async fn find_by_sku(
        pool: &PgPool,
        sku_code: &str,
    ) -> Result<Option<Product>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT id, sku_code, name, unit, spec, barcode,
                   status, created_at, updated_at
            FROM products
            WHERE sku_code = $1
            "#,
        )
        .bind(sku_code)
        .fetch_optional(pool)
        .await
    }

    /* -------------------------------------------------------------- */
    /*  Mutations                                                      */
    /* -------------------------------------------------------------- */

    /// Insert a new product. Returns the row with generated `id` and timestamps.
    pub async fn insert(
        pool: &PgPool,
        sku_code: &str,
        name: &str,
        unit: &str,
        spec: Option<&str>,
        barcode: Option<&str>,
    ) -> Result<Product, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO products (sku_code, name, unit, spec, barcode)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(sku_code)
        .bind(name)
        .bind(unit)
        .bind(spec)
        .bind(barcode)
        .fetch_one(pool)
        .await
    }

    /// Update product fields. Only the provided columns are changed.
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        sku_code: &str,
        name: &str,
        unit: &str,
        spec: Option<&str>,
        barcode: Option<&str>,
    ) -> Result<Product, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE products
            SET sku_code  = $2,
                name      = $3,
                unit      = $4,
                spec      = $5,
                barcode   = $6,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(sku_code)
        .bind(name)
        .bind(unit)
        .bind(spec)
        .bind(barcode)
        .fetch_one(pool)
        .await
    }

    /// Toggle product status (active ↔ disabled).
    pub async fn update_status(
        pool: &PgPool,
        id: Uuid,
        status: ProductStatus,
    ) -> Result<Product, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE products
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
