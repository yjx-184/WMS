import apiClient from './client';
import type { PaginatedResponse } from '../types/product';
import type {
  CreateLocationRequest,
  Location,
  LocationListQuery,
  LocationStatus,
  UpdateLocationRequest,
} from '../types/location';

/** Fetch a paginated, filterable location list for a warehouse. */
export async function listLocations(
  warehouseId: string,
  params: LocationListQuery,
): Promise<PaginatedResponse<Location>> {
  const { data } = await apiClient.get(
    `/warehouses/${warehouseId}/locations`,
    { params },
  );
  return data.data;
}

/** Create a new location within a warehouse. */
export async function createLocation(
  warehouseId: string,
  req: CreateLocationRequest,
): Promise<Location> {
  const { data } = await apiClient.post(
    `/warehouses/${warehouseId}/locations`,
    req,
  );
  return data.data;
}

/** Update an existing location. */
export async function updateLocation(
  id: string,
  req: UpdateLocationRequest,
): Promise<Location> {
  const { data } = await apiClient.put(`/locations/${id}`, req);
  return data.data;
}

/** Toggle location status (active ↔ disabled). */
export async function updateLocationStatus(
  id: string,
  status: LocationStatus,
): Promise<Location> {
  const { data } = await apiClient.patch(`/locations/${id}/status`, { status });
  return data.data;
}
