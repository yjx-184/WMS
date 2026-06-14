use crate::model::location::{Location, LocationStatus, LocationType};
use sqlx::PgPool;
use uuid::Uuid;

/// Data-access layer for the `locations` table.
///
/// All SQL is parameterised; business rules belong in the service layer.
pub struct LocationRepository;

impl LocationRepository {
    /* -------------------------------------------------------------- */
    /*  Queries                                                        */
    /* -------------------------------------------------------------- */

    /// Paginated list for a specific warehouse with optional filters.
    ///
    /// Keyword searches `code` case-insensitively.
    pub async fn list_by_warehouse(
        pool: &PgPool,
        warehouse_id: Uuid,
        keyword: Option<&str>,
        location_type: Option<LocationType>,
        status: Option<LocationStatus>,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<Location>, i64), sqlx::Error> {
        let keyword_param = keyword.map(|k| format!("%{}%", k));
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let limit = page_size as i64;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) AS count
            FROM locations
            WHERE warehouse_id = $1
              AND ($2::text IS NULL OR code ILIKE $2)
              AND ($3::location_type IS NULL OR location_type = $3)
              AND ($4::location_status IS NULL OR status = $4)
            "#,
        )
        .bind(warehouse_id)
        .bind(&keyword_param)
        .bind(&location_type)
        .bind(&status)
        .fetch_one(pool)
        .await?;

        let items: Vec<Location> = sqlx::query_as(
            r#"
            SELECT id, warehouse_id, code, location_type,
                   max_volume, max_weight, status, created_at, updated_at
            FROM locations
            WHERE warehouse_id = $1
              AND ($2::text IS NULL OR code ILIKE $2)
              AND ($3::location_type IS NULL OR location_type = $3)
              AND ($4::location_status IS NULL OR status = $4)
            ORDER BY created_at DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(warehouse_id)
        .bind(&keyword_param)
        .bind(&location_type)
        .bind(&status)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok((items, total.0))
    }

    /// Look up a single location by primary key.
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Location>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT id, warehouse_id, code, location_type,
                   max_volume, max_weight, status, created_at, updated_at
            FROM locations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// Look up a location by code within a specific warehouse
    /// (unique constraint on `warehouse_id, code`).
    pub async fn find_by_code(
        pool: &PgPool,
        warehouse_id: Uuid,
        code: &str,
    ) -> Result<Option<Location>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT id, warehouse_id, code, location_type,
                   max_volume, max_weight, status, created_at, updated_at
            FROM locations
            WHERE warehouse_id = $1 AND code = $2
            "#,
        )
        .bind(warehouse_id)
        .bind(code)
        .fetch_optional(pool)
        .await
    }

    /* -------------------------------------------------------------- */
    /*  Mutations                                                      */
    /* -------------------------------------------------------------- */

    /// Insert a new location. Returns the full row.
    pub async fn insert(
        pool: &PgPool,
        warehouse_id: Uuid,
        code: &str,
        location_type: LocationType,
    ) -> Result<Location, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO locations (warehouse_id, code, location_type)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(warehouse_id)
        .bind(code)
        .bind(&location_type)
        .fetch_one(pool)
        .await
    }

    /// Update location fields.
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        code: &str,
        location_type: LocationType,
    ) -> Result<Location, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE locations
            SET code          = $2,
                location_type = $3,
                updated_at    = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(code)
        .bind(&location_type)
        .fetch_one(pool)
        .await
    }

    /// Toggle location status (active ↔ disabled).
    pub async fn update_status(
        pool: &PgPool,
        id: Uuid,
        status: LocationStatus,
    ) -> Result<Location, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE locations
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
