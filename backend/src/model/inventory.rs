use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Row type for the `inventories` table (DDL §4.4).
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Inventory {
    pub id: Uuid,
    pub product_id: Uuid,
    pub warehouse_id: Uuid,
    pub location_id: Uuid,
    pub quantity: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Maps to the `transaction_change_type` PostgreSQL enum.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "transaction_change_type", rename_all = "snake_case")]
pub enum TransactionChangeType {
    Inbound,
    Outbound,
}

/// Row type for the `inventory_transactions` table (DDL §4.7).
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct InventoryTransaction {
    pub id: Uuid,
    pub product_id: Uuid,
    pub warehouse_id: Uuid,
    pub location_id: Uuid,
    pub change_type: TransactionChangeType,
    pub quantity: Decimal,
    pub quantity_before: Decimal,
    pub quantity_after: Decimal,
    pub reference_type: String,
    pub reference_id: Uuid,
    pub created_at: DateTime<Utc>,
}
