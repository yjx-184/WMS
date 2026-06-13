import apiClient from './client';
import type { PaginatedResponse } from '../types/product';
import type {
  CreateWarehouseRequest,
  UpdateWarehouseRequest,
  Warehouse,
  WarehouseListQuery,
  WarehouseStatus,
} from '../types/warehouse';

/** Fetch a paginated, filterable warehouse list. */
export async function listWarehouses(
  params: WarehouseListQuery,
): Promise<PaginatedResponse<Warehouse>> {
  const { data } = await apiClient.get('/warehouses', { params });
  return data.data;
}

/** Create a new warehouse. */
export async function createWarehouse(
  req: CreateWarehouseRequest,
): Promise<Warehouse> {
  const { data } = await apiClient.post('/warehouses', req);
  return data.data;
}

/** Update an existing warehouse. */
export async function updateWarehouse(
  id: string,
  req: UpdateWarehouseRequest,
): Promise<Warehouse> {
  const { data } = await apiClient.put(`/warehouses/${id}`, req);
  return data.data;
}

/** Toggle warehouse status (active ↔ disabled). */
export async function updateWarehouseStatus(
  id: string,
  status: WarehouseStatus,
): Promise<Warehouse> {
  const { data } = await apiClient.patch(`/warehouses/${id}/status`, { status });
  return data.data;
}
