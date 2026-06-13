use crate::db::AppState;
use crate::dto::product::{
    CheckSkuQuery, CreateProductRequest, ProductListQuery, UpdateProductRequest,
    UpdateProductStatusRequest,
};
use crate::error::AppError;
use crate::service::product_service::ProductService;
use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Serialize;
use serde_json::{Value, json};
use uuid::Uuid;

/* ------------------------------------------------------------------ */
/*  Response helpers                                                   */
/* ------------------------------------------------------------------ */

fn ok_body(data: impl Serialize) -> Json<Value> {
    Json(json!({"code": 0, "data": data, "message": "ok"}))
}

/* ------------------------------------------------------------------ */
/*  Handlers                                                           */
/* ------------------------------------------------------------------ */

/// `GET /api/v1/products`
pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<ProductListQuery>,
) -> Result<Json<Value>, AppError> {
    let result = ProductService::list(&state.pool, query).await?;
    Ok(ok_body(result))
}

/// `GET /api/v1/products/{id}`
pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let product = ProductService::get_by_id(&state.pool, id).await?;
    Ok(ok_body(product))
}

/// `POST /api/v1/products`
pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<Value>, AppError> {
    let product = ProductService::create(&state.pool, req).await?;
    Ok(ok_body(product))
}

/// `PUT /api/v1/products/{id}`
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProductRequest>,
) -> Result<Json<Value>, AppError> {
    let product = ProductService::update(&state.pool, id, req).await?;
    Ok(ok_body(product))
}

/// `PATCH /api/v1/products/{id}/status`
pub async fn toggle_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProductStatusRequest>,
) -> Result<Json<Value>, AppError> {
    let product = ProductService::toggle_status(&state.pool, id, req).await?;
    Ok(ok_body(product))
}

/// `GET /api/v1/products/check-sku`
pub async fn check_sku(
    State(state): State<AppState>,
    Query(query): Query<CheckSkuQuery>,
) -> Result<Json<Value>, AppError> {
    let available = ProductService::check_sku(&state.pool, query).await?;
    Ok(ok_body(json!({"available": available})))
}
