use crate::dto::location::{
    CreateLocationRequest, LocationListQuery, LocationListResponse, LocationResponse,
    UpdateLocationRequest, UpdateLocationStatusRequest,
};
use crate::error::AppError;
use crate::repository::location_repo::LocationRepository;
use crate::repository::warehouse_repo::WarehouseRepository;
use sqlx::PgPool;

pub struct LocationService;

impl LocationService {
    /// Paginated list of locations for a specific warehouse.
    pub async fn list_by_warehouse(
        pool: &PgPool,
        warehouse_id: uuid::Uuid,
        query: LocationListQuery,
    ) -> Result<LocationListResponse, AppError> {
        // Verify warehouse exists
        WarehouseRepository::find_by_id(pool, warehouse_id)
            .await?
            .ok_or_else(|| AppError::NotFound("仓库不存在".into()))?;

        let (items, total) = LocationRepository::list_by_warehouse(
            pool,
            warehouse_id,
            query.keyword.as_deref(),
            query.location_type,
            query.status,
            query.page,
            query.page_size,
        )
        .await?;

        Ok(LocationListResponse {
            items: items.into_iter().map(LocationResponse::from).collect(),
            total,
            page: query.page,
            page_size: query.page_size,
        })
    }

    /// Create a new location in a warehouse.
    ///
    /// - 404 if warehouse does not exist.
    /// - 409 if code already exists in the same warehouse.
    pub async fn create(
        pool: &PgPool,
        warehouse_id: uuid::Uuid,
        req: CreateLocationRequest,
    ) -> Result<LocationResponse, AppError> {
        // Verify warehouse exists
        WarehouseRepository::find_by_id(pool, warehouse_id)
            .await?
            .ok_or_else(|| AppError::NotFound("仓库不存在".into()))?;

        // Code uniqueness within same warehouse
        if LocationRepository::find_by_code(pool, warehouse_id, &req.code)
            .await?
            .is_some()
        {
            return Err(AppError::Conflict("库位编码已存在".into()));
        }

        let loc =
            LocationRepository::insert(pool, warehouse_id, &req.code, req.location_type).await?;

        Ok(LocationResponse::from(loc))
    }

    /// Update an existing location.
    ///
    /// - 404 if location does not exist.
    /// - 409 if the new code conflicts with another location in the same warehouse.
    pub async fn update(
        pool: &PgPool,
        id: uuid::Uuid,
        req: UpdateLocationRequest,
    ) -> Result<LocationResponse, AppError> {
        let existing = LocationRepository::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("库位不存在".into()))?;

        // If code changed, check uniqueness within the same warehouse
        if req.code != existing.code {
            if let Some(conflict) =
                LocationRepository::find_by_code(pool, existing.warehouse_id, &req.code).await?
            {
                if conflict.id != id {
                    return Err(AppError::Conflict("库位编码已存在".into()));
                }
            }
        }

        let loc = LocationRepository::update(pool, id, &req.code, req.location_type).await?;
        Ok(LocationResponse::from(loc))
    }

    /// Toggle location status (active ↔ disabled).
    pub async fn toggle_status(
        pool: &PgPool,
        id: uuid::Uuid,
        req: UpdateLocationStatusRequest,
    ) -> Result<LocationResponse, AppError> {
        LocationRepository::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("库位不存在".into()))?;

        let loc = LocationRepository::update_status(pool, id, req.status).await?;
        Ok(LocationResponse::from(loc))
    }
}
