use crate::db::AppState;
use crate::dto::outbound::{
    CompleteOutboundRequest, CreateOutboundOrderRequest, OutboundOrderListQuery,
    UpdateOutboundOrderRequest,
};
use crate::error::AppError;
use crate::service::outbound_service::OutboundService;
use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Serialize;
use serde_json::{Value, json};
use uuid::Uuid;

fn ok_body(data: impl Serialize) -> Json<Value> {
    Json(json!({"code": 0, "data": data, "message": "ok"}))
}

/// `GET /api/v1/outbound-orders`
pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<OutboundOrderListQuery>,
) -> Result<Json<Value>, AppError> {
    let result = OutboundService::list(&state.pool, query).await?;
    Ok(ok_body(result))
}

/// `GET /api/v1/outbound-orders/{id}`
pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let order = OutboundService::get_by_id(&state.pool, id).await?;
    Ok(ok_body(order))
}

/// `POST /api/v1/outbound-orders`
pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateOutboundOrderRequest>,
) -> Result<Json<Value>, AppError> {
    let order = OutboundService::create(&state.pool, req).await?;
    Ok(ok_body(order))
}

/// `PUT /api/v1/outbound-orders/{id}`
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateOutboundOrderRequest>,
) -> Result<Json<Value>, AppError> {
    let order = OutboundService::update(&state.pool, id, req).await?;
    Ok(ok_body(order))
}

/// `POST /api/v1/outbound-orders/{id}/complete`
pub async fn complete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<CompleteOutboundRequest>,
) -> Result<Json<Value>, AppError> {
    let order = OutboundService::complete(&state.pool, id, req).await?;
    Ok(ok_body(order))
}

/// `POST /api/v1/outbound-orders/{id}/cancel`
pub async fn cancel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let order = OutboundService::cancel(&state.pool, id).await?;
    Ok(ok_body(order))
}
