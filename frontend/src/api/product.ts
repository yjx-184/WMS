import apiClient from './client';
import type {
  PaginatedResponse,
  Product,
  ProductListQuery,
  ProductStatus,
} from '../types/product';

/** Fetch a paginated, filterable product list. */
export async function listProducts(
  params: ProductListQuery,
): Promise<PaginatedResponse<Product>> {
  const { data } = await apiClient.get('/products', { params });
  return data.data; // the Axios interceptor already unwrapped `code`
}

/** Toggle product status (active ↔ disabled). */
export async function updateProductStatus(
  id: string,
  status: ProductStatus,
): Promise<Product> {
  const { data } = await apiClient.patch(`/products/${id}/status`, { status });
  return data.data;
}
