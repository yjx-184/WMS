use crate::model::product::{Product, ProductStatus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/* ------------------------------------------------------------------ */
/*  Generic paginated envelope                                        */
/* ------------------------------------------------------------------ */

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}

/* ------------------------------------------------------------------ */
/*  Request DTOs                                                       */
/* ------------------------------------------------------------------ */

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub sku_code: String,
    pub name: String,
    #[serde(default = "default_unit")]
    pub unit: String,
    pub spec: Option<String>,
    pub barcode: Option<String>,
}

fn default_unit() -> String {
    "pcs".to_string()
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub sku_code: Option<String>,
    pub name: Option<String>,
    pub unit: Option<String>,
    pub spec: Option<String>,
    pub barcode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductStatusRequest {
    pub status: ProductStatus,
}

#[derive(Debug, Deserialize)]
pub struct ProductListQuery {
    pub keyword: Option<String>,
    pub status: Option<ProductStatus>,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_page() -> u32 {
    1
}
fn default_page_size() -> u32 {
    20
}

#[derive(Debug, Deserialize)]
pub struct CheckSkuQuery {
    pub sku_code: String,
    pub exclude_id: Option<Uuid>,
}

/* ------------------------------------------------------------------ */
/*  Response DTOs                                                      */
/* ------------------------------------------------------------------ */

/// Thin wrapper so every response has the same shape.
pub type ProductResponse = Product;

pub type ProductListResponse = PaginatedResponse<ProductResponse>;
