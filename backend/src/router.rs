use crate::db::AppState;
use crate::handler::{product_handler, warehouse_handler};
use axum::{Json, Router, http::StatusCode, routing::get};
use serde_json::json;

/// Stub handler — every business route returns 501 until its real handler is wired in.
async fn not_implemented() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"code": 50100, "data": null, "message": "not implemented"})),
    )
}

/// Health-check handler (identical to the previous inline version).
pub async fn health() -> Json<serde_json::Value> {
    Json(json!({"code": 0, "data": null, "message": "ok"}))
}

/// Build the full Axum router with health + 29 business-route stubs.
///
/// **Route ordering matters**: literal path segments (`check-sku`) must be
/// registered *before* parameterised segments (`{id}`) to avoid the
/// parameter capturing the literal.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // ── Health ──────────────────────────────────────────────
        .route("/api/v1/health", get(health))
        // ── Products (6) ────────────────────────────────────────
        // IMPORTANT: check-sku *before* {id}
        .route(
            "/api/v1/products",
            get(product_handler::list).post(product_handler::create),
        )
        .route(
            "/api/v1/products/check-sku",
            get(product_handler::check_sku),
        )
        .route(
            "/api/v1/products/{id}",
            get(product_handler::get_by_id).put(product_handler::update),
        )
        .route(
            "/api/v1/products/{id}/status",
            axum::routing::patch(product_handler::toggle_status),
        )
        // ── Warehouses (5) ──────────────────────────────────────
        .route(
            "/api/v1/warehouses",
            get(warehouse_handler::list).post(warehouse_handler::create),
        )
        .route(
            "/api/v1/warehouses/{id}",
            get(warehouse_handler::get_by_id).put(warehouse_handler::update),
        )
        .route(
            "/api/v1/warehouses/{id}/status",
            axum::routing::patch(warehouse_handler::toggle_status),
        )
        // ── Locations (4) ───────────────────────────────────────
        .route(
            "/api/v1/warehouses/{id}/locations",
            get(not_implemented).post(not_implemented),
        )
        .route(
            "/api/v1/locations/{id}",
            axum::routing::put(not_implemented),
        )
        .route(
            "/api/v1/locations/{id}/status",
            axum::routing::patch(not_implemented),
        )
        // ── Inbound orders (6) ──────────────────────────────────
        .route(
            "/api/v1/inbound-orders",
            get(not_implemented).post(not_implemented),
        )
        .route(
            "/api/v1/inbound-orders/{id}",
            get(not_implemented).put(not_implemented),
        )
        .route(
            "/api/v1/inbound-orders/{id}/complete",
            axum::routing::post(not_implemented),
        )
        .route(
            "/api/v1/inbound-orders/{id}/cancel",
            axum::routing::post(not_implemented),
        )
        // ── Outbound orders (6) ─────────────────────────────────
        .route(
            "/api/v1/outbound-orders",
            get(not_implemented).post(not_implemented),
        )
        .route(
            "/api/v1/outbound-orders/{id}",
            get(not_implemented).put(not_implemented),
        )
        .route(
            "/api/v1/outbound-orders/{id}/complete",
            axum::routing::post(not_implemented),
        )
        .route(
            "/api/v1/outbound-orders/{id}/cancel",
            axum::routing::post(not_implemented),
        )
        // ── Inventory (2) ───────────────────────────────────────
        .route("/api/v1/inventory", get(not_implemented))
        .route("/api/v1/inventory-transactions", get(not_implemented))
        // ── State & middleware (applied in main.rs) ─────────────
        .with_state(state)
}
