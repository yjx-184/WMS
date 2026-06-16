export type InboundOrderType = 'purchase' | 'return' | 'manual';
export type InboundOrderStatus = 'draft' | 'completed' | 'cancelled';

export interface InboundOrderListItem {
  id: string;
  order_no: string;
  warehouse_id: string;
  warehouse_name: string;
  order_type: InboundOrderType;
  status: InboundOrderStatus;
  remark: string | null;
  completed_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface InboundOrderListQuery {
  keyword?: string;
  warehouse_id?: string;
  status?: InboundOrderStatus;
  start_date?: string;
  end_date?: string;
  page: number;
  page_size: number;
}

/* ------------------------------------------------------------------ */
/*  Detail / form types                                                */
/* ------------------------------------------------------------------ */

export interface InboundOrderItemDetail {
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

export interface InboundOrderDetail {
  id: string;
  order_no: string;
  warehouse_id: string;
  warehouse_name: string;
  order_type: InboundOrderType;
  status: InboundOrderStatus;
  remark: string | null;
  completed_at: string | null;
  created_at: string;
  updated_at: string;
  items: InboundOrderItemDetail[];
}

export interface CreateInboundItemRequest {
  product_id: string;
  location_id: string;
  planned_qty: number;
}

export interface CreateInboundOrderRequest {
  warehouse_id: string;
  order_type: InboundOrderType;
  remark?: string;
  items: CreateInboundItemRequest[];
}

export interface UpdateInboundOrderRequest {
  warehouse_id: string;
  order_type: InboundOrderType;
  remark?: string;
  items: CreateInboundItemRequest[];
}

