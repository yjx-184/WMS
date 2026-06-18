use crate::db::AppState;
use crate::dto::inventory::{InventoryQueryParams, TransactionQueryParams};
use crate::error::AppError;
use crate::service::stock_query_service::StockQueryService;
use axum::Json;
use axum::extract::{Query, State};
use serde::Serialize;
use serde_json::Value;

fn ok_body(data: impl Serialize) -> Json<Value> {
    Json(serde_json::json!({"code": 0, "data": data, "message": "ok"}))
}

/// `GET /api/v1/inventory`
pub async fn query_inventory(
    State(state): State<AppState>,
    Query(params): Query<InventoryQueryParams>,
) -> Result<Json<Value>, AppError> {
    let result = StockQueryService::query_inventory(&state.pool, params).await?;
    Ok(ok_body(result))
}

/// `GET /api/v1/inventory-transactions`
pub async fn query_transactions(
    State(state): State<AppState>,
    Query(params): Query<TransactionQueryParams>,
) -> Result<Json<Value>, AppError> {
    let result = StockQueryService::query_transactions(&state.pool, params).await?;
    Ok(ok_body(result))
}
