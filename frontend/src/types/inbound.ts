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
