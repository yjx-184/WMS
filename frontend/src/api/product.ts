import apiClient from './client';
import type {
  CreateProductRequest,
  PaginatedResponse,
  Product,
  ProductListQuery,
  ProductStatus,
  UpdateProductRequest,
} from '../types/product';

/** Fetch a paginated, filterable product list. */
export async function listProducts(
  params: ProductListQuery,
): Promise<PaginatedResponse<Product>> {
  const { data } = await apiClient.get('/products', { params });
  return data.data; // the Axios interceptor already unwrapped `code`
}

/** Create a new product. */
export async function createProduct(req: CreateProductRequest): Promise<Product> {
  const { data } = await apiClient.post('/products', req);
  return data.data;
}

/** Update an existing product. */
export async function updateProduct(
  id: string,
  req: UpdateProductRequest,
): Promise<Product> {
  const { data } = await apiClient.put(`/products/${id}`, req);
  return data.data;
}

/** Toggle product status (active ↔ disabled). */
export async function updateProductStatus(
  id: string,
  status: ProductStatus,
): Promise<Product> {
  const { data } = await apiClient.patch(`/products/${id}/status`, { status });
  return data.data;
}

/** Check whether a SKU code is available. */
export async function checkSku(
  sku_code: string,
  exclude_id?: string,
): Promise<boolean> {
  const { data } = await apiClient.get('/products/check-sku', {
    params: { sku_code, exclude_id: exclude_id || undefined },
  });
  return data.data.available;
}
