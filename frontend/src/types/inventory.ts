export interface InventoryRow {
  id: string;
  product_id: string;
  product_name: string;
  sku_code: string;
  warehouse_id: string;
  warehouse_name: string;
  location_id: string;
  location_code: string;
  quantity: string;
  created_at: string;
  updated_at: string;
}

export interface InventoryQueryParams {
  product_id?: string;
  warehouse_id?: string;
  location_id?: string;
  keyword?: string;
  page: number;
  page_size: number;
}
