export type OutboundOrderType = 'sales' | 'manual' | 'scrap';
export type OutboundOrderStatus = 'draft' | 'completed' | 'cancelled';

export interface OutboundOrderListItem {
  id: string;
  order_no: string;
  warehouse_id: string;
  warehouse_name: string;
  order_type: OutboundOrderType;
  status: OutboundOrderStatus;
  remark: string | null;
  completed_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface OutboundOrderListQuery {
  keyword?: string;
  warehouse_id?: string;
  status?: OutboundOrderStatus;
  start_date?: string;
  end_date?: string;
  page: number;
  page_size: number;
}

/* ------------------------------------------------------------------ */
/*  Detail / form types                                                */
/* ------------------------------------------------------------------ */

export interface OutboundOrderItemDetail {
  id: string;
  product_id: string;
  product_name: string;
  sku_code: string;
  location_id: string;
  location_code: string;
  planned_qty: string;
  actual_qty: string | null;
  created_at: string;
}

export interface OutboundOrderDetail {
  id: string;
  order_no: string;
  warehouse_id: string;
  warehouse_name: string;
  order_type: OutboundOrderType;
  status: OutboundOrderStatus;
  remark: string | null;
  completed_at: string | null;
  created_at: string;
  updated_at: string;
  items: OutboundOrderItemDetail[];
}

export interface CreateOutboundItemRequest {
  product_id: string;
  location_id: string;
  planned_qty: number;
}

export interface CreateOutboundOrderRequest {
  warehouse_id: string;
  order_type: OutboundOrderType;
  remark?: string;
  items: CreateOutboundItemRequest[];
}

export interface UpdateOutboundOrderRequest {
  warehouse_id: string;
  order_type: OutboundOrderType;
  remark?: string;
  items: CreateOutboundItemRequest[];
}

export interface CompleteOutboundItemRequest {
  item_id: string;
  actual_qty: number;
}

export interface CompleteOutboundRequest {
  items: CompleteOutboundItemRequest[];
}


