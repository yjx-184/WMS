import apiClient from './client';
import type { PaginatedResponse } from '../types/product';
import type { InboundOrderListItem, InboundOrderListQuery } from '../types/inbound';

export async function listInboundOrders(
  params: InboundOrderListQuery,
): Promise<PaginatedResponse<InboundOrderListItem>> {
  const { data } = await apiClient.get('/inbound-orders', { params });
  return data.data;
}


