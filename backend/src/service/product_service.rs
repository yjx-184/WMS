use crate::dto::product::{
    CheckSkuQuery, CreateProductRequest, ProductListQuery, ProductListResponse,
    UpdateProductRequest, UpdateProductStatusRequest,
};
use crate::error::AppError;
use crate::model::product::Product;
use crate::repository::product_repo::ProductRepository;
use sqlx::PgPool;

pub struct ProductService;

impl ProductService {
    /// Paginated list with keyword / status filters.
    pub async fn list(
        pool: &PgPool,
        query: ProductListQuery,
    ) -> Result<ProductListResponse, AppError> {
        let (items, total) = ProductRepository::list(
            pool,
            query.keyword.as_deref(),
            query.status,
            query.page,
            query.page_size,
        )
        .await?;

        Ok(ProductListResponse {
            items,
            total,
            page: query.page,
            page_size: query.page_size,
        })
    }

    /// Get a single product by id.
    pub async fn get_by_id(pool: &PgPool, id: uuid::Uuid) -> Result<Product, AppError> {
        ProductRepository::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("商品不存在".into()))
    }

    /// Create a new product. Returns 409 if the SKU already exists.
    pub async fn create(pool: &PgPool, req: CreateProductRequest) -> Result<Product, AppError> {
        // SKU uniqueness check
        if ProductRepository::find_by_sku(pool, &req.sku_code)
            .await?
            .is_some()
        {
            return Err(AppError::Conflict("SKU编码已存在".into()));
        }

        let product = ProductRepository::insert(
            pool,
            &req.sku_code,
            &req.name,
            &req.unit,
            req.spec.as_deref(),
            req.barcode.as_deref(),
        )
        .await?;

        Ok(product)
    }

    /// Update an existing product.
    ///
    /// - 404 if the product does not exist.
    /// - 409 if the new SKU conflicts with another product.
    pub async fn update(
        pool: &PgPool,
        id: uuid::Uuid,
        req: UpdateProductRequest,
    ) -> Result<Product, AppError> {
        let existing = ProductRepository::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("商品不存在".into()))?;

        // If the SKU changed, verify it is not already taken by another product.
        if req.sku_code != existing.sku_code {
            if let Some(conflict) = ProductRepository::find_by_sku(pool, &req.sku_code).await? {
                if conflict.id != id {
                    return Err(AppError::Conflict("SKU编码已存在".into()));
                }
            }
        }

        let product = ProductRepository::update(
            pool,
            id,
            &req.sku_code,
            &req.name,
            &req.unit,
            req.spec.as_deref(),
            req.barcode.as_deref(),
        )
        .await?;

        Ok(product)
    }

    /// Toggle product status (active ↔ disabled).
    pub async fn toggle_status(
        pool: &PgPool,
        id: uuid::Uuid,
        req: UpdateProductStatusRequest,
    ) -> Result<Product, AppError> {
        // Verify the product exists
        ProductRepository::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("商品不存在".into()))?;

        let product = ProductRepository::update_status(pool, id, req.status).await?;
        Ok(product)
    }

    /// Check whether a SKU code is available.
    ///
    /// When `exclude_id` is provided the product with that id is ignored
    /// (so editing a product does not flag its own SKU as taken).
    pub async fn check_sku(pool: &PgPool, query: CheckSkuQuery) -> Result<bool, AppError> {
        let existing = ProductRepository::find_by_sku(pool, &query.sku_code).await?;

        match existing {
            None => Ok(true),
            Some(p) => {
                if let Some(exclude) = query.exclude_id {
                    Ok(p.id == exclude)
                } else {
                    Ok(false)
                }
            }
        }
    }
}
