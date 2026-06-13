use crate::db::AppState;
use crate::dto::warehouse::{
    CreateWarehouseRequest, UpdateWarehouseRequest, UpdateWarehouseStatusRequest,
    WarehouseListQuery,
};
use crate::error::AppError;
use crate::service::warehouse_service::WarehouseService;
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

/// `GET /api/v1/warehouses`
pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<WarehouseListQuery>,
) -> Result<Json<Value>, AppError> {
    let result = WarehouseService::list(&state.pool, query).await?;
    Ok(ok_body(result))
}

/// `GET /api/v1/warehouses/{id}`
pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let warehouse = WarehouseService::get_by_id(&state.pool, id).await?;
    Ok(ok_body(warehouse))
}

/// `POST /api/v1/warehouses`
pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateWarehouseRequest>,
) -> Result<Json<Value>, AppError> {
    let warehouse = WarehouseService::create(&state.pool, req).await?;
    Ok(ok_body(warehouse))
}

/// `PUT /api/v1/warehouses/{id}`
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateWarehouseRequest>,
) -> Result<Json<Value>, AppError> {
    let warehouse = WarehouseService::update(&state.pool, id, req).await?;
    Ok(ok_body(warehouse))
}

/// `PATCH /api/v1/warehouses/{id}/status`
pub async fn toggle_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateWarehouseStatusRequest>,
) -> Result<Json<Value>, AppError> {
    let warehouse = WarehouseService::toggle_status(&state.pool, id, req).await?;
    Ok(ok_body(warehouse))
}
