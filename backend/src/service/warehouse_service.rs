use crate::dto::warehouse::{
    CreateWarehouseRequest, UpdateWarehouseRequest, UpdateWarehouseStatusRequest,
    WarehouseListQuery, WarehouseListResponse,
};
use crate::error::AppError;
use crate::model::warehouse::Warehouse;
use crate::repository::warehouse_repo::WarehouseRepository;
use sqlx::PgPool;

pub struct WarehouseService;

impl WarehouseService {
    /// Paginated list with keyword / status filters.
    pub async fn list(
        pool: &PgPool,
        query: WarehouseListQuery,
    ) -> Result<WarehouseListResponse, AppError> {
        let (items, total) = WarehouseRepository::list(
            pool,
            query.keyword.as_deref(),
            query.status,
            query.page,
            query.page_size,
        )
        .await?;

        Ok(WarehouseListResponse {
            items,
            total,
            page: query.page,
            page_size: query.page_size,
        })
    }

    /// Get a single warehouse by id.
    pub async fn get_by_id(pool: &PgPool, id: uuid::Uuid) -> Result<Warehouse, AppError> {
        WarehouseRepository::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("仓库不存在".into()))
    }

    /// Create a new warehouse. Returns 409 if the code already exists.
    pub async fn create(pool: &PgPool, req: CreateWarehouseRequest) -> Result<Warehouse, AppError> {
        if WarehouseRepository::find_by_code(pool, &req.code)
            .await?
            .is_some()
        {
            return Err(AppError::Conflict("仓库编码已存在".into()));
        }

        let warehouse =
            WarehouseRepository::insert(pool, &req.code, &req.name, req.address.as_deref()).await?;

        Ok(warehouse)
    }

    /// Update an existing warehouse.
    ///
    /// - 404 if the warehouse does not exist.
    /// - 409 if the new code conflicts with another warehouse.
    pub async fn update(
        pool: &PgPool,
        id: uuid::Uuid,
        req: UpdateWarehouseRequest,
    ) -> Result<Warehouse, AppError> {
        let existing = WarehouseRepository::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("仓库不存在".into()))?;

        // If the code changed, verify it is not already taken by another warehouse.
        if req.code != existing.code {
            if let Some(conflict) = WarehouseRepository::find_by_code(pool, &req.code).await? {
                if conflict.id != id {
                    return Err(AppError::Conflict("仓库编码已存在".into()));
                }
            }
        }

        let warehouse =
            WarehouseRepository::update(pool, id, &req.code, &req.name, req.address.as_deref())
                .await?;

        Ok(warehouse)
    }

    /// Toggle warehouse status (active ↔ disabled).
    pub async fn toggle_status(
        pool: &PgPool,
        id: uuid::Uuid,
        req: UpdateWarehouseStatusRequest,
    ) -> Result<Warehouse, AppError> {
        WarehouseRepository::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("仓库不存在".into()))?;

        let warehouse = WarehouseRepository::update_status(pool, id, req.status).await?;
        Ok(warehouse)
    }
}
