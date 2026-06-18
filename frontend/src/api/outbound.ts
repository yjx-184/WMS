import apiClient from './client';
import type { PaginatedResponse } from '../types/product';
import type { OutboundOrderListItem, OutboundOrderListQuery } from '../types/outbound';

export async function listOutboundOrders(
  params: OutboundOrderListQuery,
): Promise<PaginatedResponse<OutboundOrderListItem>> {
  const { data } = await apiClient.get('/outbound-orders', { params });
  return data.data;
}
