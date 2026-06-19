import apiClient from './client';
import type { PaginatedResponse } from '../types/product';
import type { InventoryQueryParams, InventoryRow } from '../types/inventory';

export async function queryInventory(
  params: InventoryQueryParams,
): Promise<PaginatedResponse<InventoryRow>> {
  const { data } = await apiClient.get('/inventory', { params });
  return data.data;
}
