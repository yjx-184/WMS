export type WarehouseStatus = 'active' | 'disabled';

export interface Warehouse {
  id: string;
  code: string;
  name: string;
  address: string | null;
  status: WarehouseStatus;
  created_at: string;
  updated_at: string;
}

export interface WarehouseListQuery {
  keyword?: string;
  status?: WarehouseStatus;
  page: number;
  page_size: number;
}

export interface CreateWarehouseRequest {
  code: string;
  name: string;
  address?: string;
}

export interface UpdateWarehouseRequest {
  code: string;
  name: string;
  address?: string;
}
