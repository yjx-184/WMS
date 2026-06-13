use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Maps to the `product_status` PostgreSQL enum.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "product_status", rename_all = "snake_case")]
pub enum ProductStatus {
    Active,
    Disabled,
}

/// Row type for the `products` table.
///
/// Fields mirror the DDL in `003-database-design.md` §4.1.
/// **No `category_id`** — the table has no category foreign key.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Product {
    pub id: Uuid,
    pub sku_code: String,
    pub name: String,
    pub unit: String,
    pub spec: Option<String>,
    pub barcode: Option<String>,
    pub status: ProductStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
