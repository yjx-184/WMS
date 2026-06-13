use crate::dto::product::PaginatedResponse;
use crate::model::warehouse::{Warehouse, WarehouseStatus};
use serde::Deserialize;

/* ------------------------------------------------------------------ */
/*  Request DTOs                                                       */
/* ------------------------------------------------------------------ */

#[derive(Debug, Deserialize)]
pub struct CreateWarehouseRequest {
    pub code: String,
    pub name: String,
    pub address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWarehouseRequest {
    pub code: String,
    pub name: String,
    pub address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWarehouseStatusRequest {
    pub status: WarehouseStatus,
}

#[derive(Debug, Deserialize)]
pub struct WarehouseListQuery {
    pub keyword: Option<String>,
    pub status: Option<WarehouseStatus>,
    #[serde(default = "super::product::default_page")]
    pub page: u32,
    #[serde(default = "super::product::default_page_size")]
    pub page_size: u32,
}

/* ------------------------------------------------------------------ */
/*  Response DTOs                                                      */
/* ------------------------------------------------------------------ */

pub type WarehouseResponse = Warehouse;

pub type WarehouseListResponse = PaginatedResponse<WarehouseResponse>;
