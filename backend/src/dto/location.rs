use crate::dto::product::PaginatedResponse;
use crate::model::location::{Location, LocationStatus, LocationType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/* ------------------------------------------------------------------ */
/*  Request DTOs                                                       */
/* ------------------------------------------------------------------ */

#[derive(Debug, Deserialize)]
pub struct CreateLocationRequest {
    pub code: String,
    #[serde(default)]
    pub location_type: LocationType,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLocationRequest {
    pub code: String,
    pub location_type: LocationType,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLocationStatusRequest {
    pub status: LocationStatus,
}

#[derive(Debug, Deserialize)]
pub struct LocationListQuery {
    pub keyword: Option<String>,
    pub location_type: Option<LocationType>,
    pub status: Option<LocationStatus>,
    #[serde(default = "super::product::default_page")]
    pub page: u32,
    #[serde(default = "super::product::default_page_size")]
    pub page_size: u32,
}

/* ------------------------------------------------------------------ */
/*  Response DTOs  (max_volume / max_weight deliberately excluded)     */
/* ------------------------------------------------------------------ */

#[derive(Debug, Serialize)]
pub struct LocationResponse {
    pub id: Uuid,
    pub warehouse_id: Uuid,
    pub code: String,
    pub location_type: LocationType,
    pub status: LocationStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Location> for LocationResponse {
    fn from(loc: Location) -> Self {
        Self {
            id: loc.id,
            warehouse_id: loc.warehouse_id,
            code: loc.code,
            location_type: loc.location_type,
            status: loc.status,
            created_at: loc.created_at,
            updated_at: loc.updated_at,
        }
    }
}

pub type LocationListResponse = PaginatedResponse<LocationResponse>;
