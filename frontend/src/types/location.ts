export type LocationType = 'normal' | 'receiving' | 'shipping' | 'return';
export type LocationStatus = 'active' | 'disabled';

export interface Location {
  id: string;
  warehouse_id: string;
  code: string;
  location_type: LocationType;
  status: LocationStatus;
  created_at: string;
  updated_at: string;
}

export interface LocationListQuery {
  keyword?: string;
  location_type?: LocationType;
  status?: LocationStatus;
  page: number;
  page_size: number;
}

export interface CreateLocationRequest {
  code: string;
  location_type?: LocationType;
}

export interface UpdateLocationRequest {
  code: string;
  location_type: LocationType;
}
