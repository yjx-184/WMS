use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Maps to the `outbound_order_type` PostgreSQL enum.
#[derive(Debug, Clone, Default, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "outbound_order_type", rename_all = "snake_case")]
pub enum OutboundOrderType {
    #[default]
    Sales,
    Manual,
    Scrap,
}

/// Maps to the `outbound_order_status` PostgreSQL enum.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "outbound_order_status", rename_all = "snake_case")]
pub enum OutboundOrderStatus {
    #[default]
    Draft,
    Completed,
    Cancelled,
}

/// Row type for the `outbound_orders` table (DDL §4.6).
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct OutboundOrder {
    pub id: Uuid,
    pub order_no: String,
    pub warehouse_id: Uuid,
    pub order_type: OutboundOrderType,
    pub status: OutboundOrderStatus,
    pub remark: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Row type for the `outbound_order_items` table (DDL §4.6).
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct OutboundOrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub location_id: Uuid,
    pub planned_qty: rust_decimal::Decimal,
    pub actual_qty: Option<rust_decimal::Decimal>,
    pub created_at: DateTime<Utc>,
}
