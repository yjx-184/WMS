import apiClient from './client';
import type { PaginatedResponse } from '../types/product';
import type {
  CreateInboundOrderRequest,
  InboundOrderDetail,
  InboundOrderListItem,
  InboundOrderListQuery,
  UpdateInboundOrderRequest,
} from '../types/inbound';

export async function listInboundOrders(
  params: InboundOrderListQuery,
): Promise<PaginatedResponse<InboundOrderListItem>> {
  const { data } = await apiClient.get('/inbound-orders', { params });
  return data.data;
}

export async function getInboundOrder(id: string): Promise<InboundOrderDetail> {
  const { data } = await apiClient.get(`/inbound-orders/${id}`);
  return data.data;
}

export async function createInboundOrder(
  req: CreateInboundOrderRequest,
): Promise<InboundOrderDetail> {
  const { data } = await apiClient.post('/inbound-orders', req);
  return data.data;
}

export async function updateInboundOrder(
  id: string,
  req: UpdateInboundOrderRequest,
): Promise<InboundOrderDetail> {
  const { data } = await apiClient.put(`/inbound-orders/${id}`, req);
  return data.data;
}

export async function completeInboundOrder(
  id: string,
  req: { items: { item_id: string; actual_qty: number }[] },
): Promise<InboundOrderDetail> {
  const { data } = await apiClient.post(`/inbound-orders/${id}/complete`, req);
  return data.data;
}

export async function cancelInboundOrder(id: string): Promise<InboundOrderDetail> {
  const { data } = await apiClient.post(`/inbound-orders/${id}/cancel`);
  return data.data;
}
