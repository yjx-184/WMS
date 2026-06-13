use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Maps to the `warehouse_status` PostgreSQL enum.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "warehouse_status", rename_all = "snake_case")]
pub enum WarehouseStatus {
    Active,
    Disabled,
}

/// Row type for the `warehouses` table.
///
/// Fields mirror the DDL in `003-database-design.md` §4.2.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Warehouse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub address: Option<String>,
    pub status: WarehouseStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
