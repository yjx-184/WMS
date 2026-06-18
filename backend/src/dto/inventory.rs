use crate::dto::product::PaginatedResponse;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/* ------------------------------------------------------------------ */
/*  Inventory query                                                    */
/* ------------------------------------------------------------------ */

#[derive(Debug, Deserialize)]
pub struct InventoryQueryParams {
    pub product_id: Option<Uuid>,
    pub warehouse_id: Option<Uuid>,
    pub location_id: Option<Uuid>,
    pub keyword: Option<String>,
    #[serde(default = "super::product::default_page")]
    pub page: u32,
    #[serde(default = "super::product::default_page_size")]
    pub page_size: u32,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct InventoryRow {
    pub id: Uuid,
    pub product_id: Uuid,
    pub product_name: String,
    pub sku_code: String,
    pub warehouse_id: Uuid,
    pub warehouse_name: String,
    pub location_id: Uuid,
    pub location_code: String,
    pub quantity: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub type InventoryListResponse = PaginatedResponse<InventoryRow>;

/* ------------------------------------------------------------------ */
/*  Transaction query                                                  */
/* ------------------------------------------------------------------ */

#[derive(Debug, Deserialize)]
pub struct TransactionQueryParams {
    pub product_id: Option<Uuid>,
    pub warehouse_id: Option<Uuid>,
    pub location_id: Option<Uuid>,
    pub change_type: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    #[serde(default = "super::product::default_page")]
    pub page: u32,
    #[serde(default = "super::product::default_page_size")]
    pub page_size: u32,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TransactionRow {
    pub id: Uuid,
    pub product_id: Uuid,
    pub product_name: String,
    pub sku_code: String,
    pub warehouse_id: Uuid,
    pub warehouse_name: String,
    pub location_id: Uuid,
    pub location_code: String,
    pub change_type: String,
    pub quantity: Decimal,
    pub quantity_before: Decimal,
    pub quantity_after: Decimal,
    pub reference_type: String,
    pub reference_id: Uuid,
    pub created_at: DateTime<Utc>,
}

pub type TransactionListResponse = PaginatedResponse<TransactionRow>;
