use crate::dto::product::PaginatedResponse;
use crate::model::inbound::{InboundOrderStatus, InboundOrderType};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/* ------------------------------------------------------------------ */
/*  Request DTOs                                                       */
/* ------------------------------------------------------------------ */

#[derive(Debug, Deserialize)]
pub struct CreateInboundItemRequest {
    pub product_id: Uuid,
    pub location_id: Uuid,
    pub planned_qty: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct CreateInboundOrderRequest {
    pub warehouse_id: Uuid,
    #[serde(default)]
    pub order_type: InboundOrderType,
    pub remark: Option<String>,
    pub items: Vec<CreateInboundItemRequest>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInboundItemRequest {
    pub product_id: Uuid,
    pub location_id: Uuid,
    pub planned_qty: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInboundOrderRequest {
    pub warehouse_id: Uuid,
    pub order_type: InboundOrderType,
    pub remark: Option<String>,
    /// Full replacement of items.
    pub items: Vec<UpdateInboundItemRequest>,
}

#[derive(Debug, Deserialize)]
pub struct CompleteInboundItemRequest {
    pub item_id: Uuid,
    pub actual_qty: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct CompleteInboundRequest {
    pub items: Vec<CompleteInboundItemRequest>,
}

#[derive(Debug, Deserialize)]
pub struct InboundOrderListQuery {
    pub keyword: Option<String>,
    pub warehouse_id: Option<Uuid>,
    pub status: Option<InboundOrderStatus>,
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

/// Row used in the paginated list — no nested items, includes warehouse name.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct InboundOrderListItem {
    pub id: Uuid,
    pub order_no: String,
    pub warehouse_id: Uuid,
    pub warehouse_name: String,
    pub order_type: InboundOrderType,
    pub status: InboundOrderStatus,
    pub remark: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single line inside the detail response — includes joined names.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct InboundOrderDetailItem {
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

/// Full detail response returned by `GET /api/v1/inbound-orders/{id}`.
#[derive(Debug, Serialize)]
pub struct InboundOrderDetailResponse {
    pub id: Uuid,
    pub order_no: String,
    pub warehouse_id: Uuid,
    pub warehouse_name: String,
    pub order_type: InboundOrderType,
    pub status: InboundOrderStatus,
    pub remark: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub items: Vec<InboundOrderDetailItem>,
}

pub type InboundOrderListResponse = PaginatedResponse<InboundOrderListItem>;
