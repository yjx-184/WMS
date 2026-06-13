/** Shared types for the Product domain. */

export type ProductStatus = 'active' | 'disabled';

export interface Product {
  id: string;
  sku_code: string;
  name: string;
  unit: string;
  spec: string | null;
  barcode: string | null;
  status: ProductStatus;
  created_at: string;
  updated_at: string;
}

export interface ProductListQuery {
  keyword?: string;
  status?: ProductStatus;
  page: number;
  page_size: number;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
}

export interface CreateProductRequest {
  sku_code: string;
  name: string;
  unit: string;
  spec?: string;
  barcode?: string;
}

export interface UpdateProductRequest {
  sku_code: string;
  name: string;
  unit: string;
  spec?: string;
  barcode?: string;
}

