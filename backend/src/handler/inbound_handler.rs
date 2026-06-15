use crate::db::AppState;
use crate::dto::inbound::{
    CompleteInboundRequest, CreateInboundOrderRequest, InboundOrderListQuery,
    UpdateInboundOrderRequest,
};
use crate::error::AppError;
use crate::service::inbound_service::InboundService;
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

/// `GET /api/v1/inbound-orders`
pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<InboundOrderListQuery>,
) -> Result<Json<Value>, AppError> {
    let result = InboundService::list(&state.pool, query).await?;
    Ok(ok_body(result))
}

/// `GET /api/v1/inbound-orders/{id}`
pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let order = InboundService::get_by_id(&state.pool, id).await?;
    Ok(ok_body(order))
}

/// `POST /api/v1/inbound-orders`
pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateInboundOrderRequest>,
) -> Result<Json<Value>, AppError> {
    let order = InboundService::create(&state.pool, req).await?;
    Ok(ok_body(order))
}

/// `PUT /api/v1/inbound-orders/{id}`
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateInboundOrderRequest>,
) -> Result<Json<Value>, AppError> {
    let order = InboundService::update(&state.pool, id, req).await?;
    Ok(ok_body(order))
}

/// `POST /api/v1/inbound-orders/{id}/complete`
pub async fn complete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<CompleteInboundRequest>,
) -> Result<Json<Value>, AppError> {
    let order = InboundService::complete(&state.pool, id, req).await?;
    Ok(ok_body(order))
}

/// `POST /api/v1/inbound-orders/{id}/cancel`
pub async fn cancel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let order = InboundService::cancel(&state.pool, id).await?;
    Ok(ok_body(order))
}
