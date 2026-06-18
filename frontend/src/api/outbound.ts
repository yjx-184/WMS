import apiClient from './client';
import type { PaginatedResponse } from '../types/product';
import type {
  CreateOutboundOrderRequest,
  OutboundOrderDetail,
  OutboundOrderListItem,
  OutboundOrderListQuery,
  UpdateOutboundOrderRequest,
} from '../types/outbound';

export async function listOutboundOrders(
  params: OutboundOrderListQuery,
): Promise<PaginatedResponse<OutboundOrderListItem>> {
  const { data } = await apiClient.get('/outbound-orders', { params });
  return data.data;
}

export async function getOutboundOrder(id: string): Promise<OutboundOrderDetail> {
  const { data } = await apiClient.get(`/outbound-orders/${id}`);
  return data.data;
}

export async function createOutboundOrder(
  req: CreateOutboundOrderRequest,
): Promise<OutboundOrderDetail> {
  const { data } = await apiClient.post('/outbound-orders', req);
  return data.data;
}

export async function updateOutboundOrder(
  id: string,
  req: UpdateOutboundOrderRequest,
): Promise<OutboundOrderDetail> {
  const { data } = await apiClient.put(`/outbound-orders/${id}`, req);
  return data.data;
}
