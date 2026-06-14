use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Maps to the `location_type` PostgreSQL enum.
#[derive(Debug, Clone, Default, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "location_type", rename_all = "snake_case")]
pub enum LocationType {
    #[default]
    Normal,
    Receiving,
    Shipping,
    Return,
}

/// Maps to the `location_status` PostgreSQL enum.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "location_status", rename_all = "snake_case")]
pub enum LocationStatus {
    Active,
    Disabled,
}

/// Row type for the `locations` table.
///
/// Fields mirror the DDL in `003-database-design.md` §4.3.
/// `max_volume` and `max_weight` are reserved DB columns; they are
/// **not** exposed to clients via the DTO layer.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Location {
    pub id: Uuid,
    pub warehouse_id: Uuid,
    pub code: String,
    pub location_type: LocationType,
    pub max_volume: Option<rust_decimal::Decimal>,
    pub max_weight: Option<rust_decimal::Decimal>,
    pub status: LocationStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
