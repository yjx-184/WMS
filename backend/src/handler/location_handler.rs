use crate::db::AppState;
use crate::dto::location::{
    CreateLocationRequest, LocationListQuery, UpdateLocationRequest, UpdateLocationStatusRequest,
};
use crate::error::AppError;
use crate::service::location_service::LocationService;
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

/// `GET /api/v1/warehouses/{warehouse_id}/locations`
pub async fn list(
    State(state): State<AppState>,
    Path(warehouse_id): Path<Uuid>,
    Query(query): Query<LocationListQuery>,
) -> Result<Json<Value>, AppError> {
    let result = LocationService::list_by_warehouse(&state.pool, warehouse_id, query).await?;
    Ok(ok_body(result))
}

/// `POST /api/v1/warehouses/{warehouse_id}/locations`
pub async fn create(
    State(state): State<AppState>,
    Path(warehouse_id): Path<Uuid>,
    Json(req): Json<CreateLocationRequest>,
) -> Result<Json<Value>, AppError> {
    let loc = LocationService::create(&state.pool, warehouse_id, req).await?;
    Ok(ok_body(loc))
}

/// `PUT /api/v1/locations/{id}`
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateLocationRequest>,
) -> Result<Json<Value>, AppError> {
    let loc = LocationService::update(&state.pool, id, req).await?;
    Ok(ok_body(loc))
}

/// `PATCH /api/v1/locations/{id}/status`
pub async fn toggle_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateLocationStatusRequest>,
) -> Result<Json<Value>, AppError> {
    let loc = LocationService::toggle_status(&state.pool, id, req).await?;
    Ok(ok_body(loc))
}
