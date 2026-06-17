use crate::dto::product::PaginatedResponse;
use crate::model::outbound::{OutboundOrderStatus, OutboundOrderType};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/* ------------------------------------------------------------------ */
/*  Request DTOs                                                       */
/* ------------------------------------------------------------------ */

#[derive(Debug, Deserialize)]
pub struct CreateOutboundItemRequest {
    pub product_id: Uuid,
    pub location_id: Uuid,
    pub planned_qty: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct CreateOutboundOrderRequest {
    pub warehouse_id: Uuid,
    #[serde(default)]
    pub order_type: OutboundOrderType,
    pub remark: Option<String>,
    pub items: Vec<CreateOutboundItemRequest>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOutboundItemRequest {
    pub product_id: Uuid,
    pub location_id: Uuid,
    pub planned_qty: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOutboundOrderRequest {
    pub warehouse_id: Uuid,
    pub order_type: OutboundOrderType,
    pub remark: Option<String>,
    /// Full replacement of items.
    pub items: Vec<UpdateOutboundItemRequest>,
}

#[derive(Debug, Deserialize)]
pub struct CompleteOutboundItemRequest {
    pub item_id: Uuid,
    pub actual_qty: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct CompleteOutboundRequest {
    pub items: Vec<CompleteOutboundItemRequest>,
}

#[derive(Debug, Deserialize)]
pub struct OutboundOrderListQuery {
    pub keyword: Option<String>,
    pub warehouse_id: Option<Uuid>,
    pub status: Option<OutboundOrderStatus>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    #[serde(default = "super::product::default_page")]
    pub page: u32,
    #[serde(default = "super::product::default_page_size")]
    pub page_size: u32,
}

/* ------------------------------------------------------------------ */
/*  Response DTOs                                                      */
/* ------------------------------------------------------------------ */

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct OutboundOrderListItem {
    pub id: Uuid,
    pub order_no: String,
    pub warehouse_id: Uuid,
    pub warehouse_name: String,
    pub order_type: OutboundOrderType,
    pub status: OutboundOrderStatus,
    pub remark: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct OutboundOrderDetailItem {
    pub id: Uuid,
    pub product_id: Uuid,
    pub product_name: String,
    pub sku_code: String,
    pub location_id: Uuid,
    pub location_code: String,
    pub planned_qty: Decimal,
    pub actual_qty: Option<Decimal>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct OutboundOrderDetailResponse {
    pub id: Uuid,
    pub order_no: String,
    pub warehouse_id: Uuid,
    pub warehouse_name: String,
    pub order_type: OutboundOrderType,
    pub status: OutboundOrderStatus,
    pub remark: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub items: Vec<OutboundOrderDetailItem>,
}

pub type OutboundOrderListResponse = PaginatedResponse<OutboundOrderListItem>;
