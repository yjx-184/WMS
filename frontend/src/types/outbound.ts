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
