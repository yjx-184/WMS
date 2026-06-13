import { useState } from 'react';
import {
  Button,
  Input,
  Select,
  Space,
  Table,
  Tag,
  type TablePaginationConfig,
} from 'antd';
import { PlusOutlined, SearchOutlined } from '@ant-design/icons';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { listProducts, updateProductStatus } from '../api/product';
import ProductFormModal from './ProductFormModal';
import type { Product, ProductListQuery, ProductStatus } from '../types/product';

const STATUS_LABELS: Record<ProductStatus, string> = {
  active: '启用',
  disabled: '禁用',
};

const STATUS_OPTIONS: { value: ProductStatus | ''; label: string }[] = [
  { value: '', label: '全部' },
  { value: 'active', label: '启用' },
  { value: 'disabled', label: '禁用' },
];

export default function ProductList() {
  const queryClient = useQueryClient();

  /* ----- modal state ----- */
  const [modalOpen, setModalOpen] = useState(false);
  const [editingProduct, setEditingProduct] = useState<Product | null>(null);

  function openCreateModal() {
    setEditingProduct(null);
    setModalOpen(true);
  }

  function openEditModal(product: Product) {
    setEditingProduct(product);
    setModalOpen(true);
  }

  function closeModal() {
    setModalOpen(false);
    setEditingProduct(null);
  }

  /* ----- search state ----- */
  const [keyword, setKeyword] = useState('');
  const [status, setStatus] = useState<ProductStatus | ''>('');
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);

  const queryParams: ProductListQuery = {
    keyword: keyword || undefined,
    status: (status || undefined) as ProductStatus | undefined,
    page,
    page_size: pageSize,
  };

  /* ----- data ----- */
  const { data, isLoading } = useQuery({
    queryKey: ['products', queryParams],
    queryFn: () => listProducts(queryParams),
  });

  /* ----- status toggle mutation ----- */
  const toggleMutation = useMutation({
    mutationFn: ({ id, next }: { id: string; next: ProductStatus }) =>
      updateProductStatus(id, next),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['products'] });
    },
  });

  function handleToggle(product: Product) {
    const next: ProductStatus =
      product.status === 'active' ? 'disabled' : 'active';
    toggleMutation.mutate({ id: product.id, next });
  }

  /* ----- search ----- */
  function handleSearch() {
    setPage(1); // reset to first page
    // query will re-fetch because queryParams changed
  }

  function handleTableChange(pagination: TablePaginationConfig) {
    if (pagination.current) setPage(pagination.current);
    if (pagination.pageSize) setPageSize(pagination.pageSize);
  }

  /* ----- columns ----- */
  const columns = [
    {
      title: 'SKU',
      dataIndex: 'sku_code',
      key: 'sku_code',
    },
    {
      title: '名称',
      dataIndex: 'name',
      key: 'name',
    },
    {
      title: '单位',
      dataIndex: 'unit',
      key: 'unit',
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      render: (s: ProductStatus) => (
        <Tag color={s === 'active' ? 'green' : 'red'}>
          {STATUS_LABELS[s]}
        </Tag>
      ),
    },
    {
      title: '操作',
      key: 'actions',
      render: (_: unknown, record: Product) => (
        <Space>
          <Button
            type="link"
            size="small"
            onClick={() => openEditModal(record)}
          >
            编辑
          </Button>
          <Button
            type="link"
            size="small"
            danger={record.status === 'active'}
            loading={toggleMutation.isPending}
            onClick={() => handleToggle(record)}
          >
            {record.status === 'active' ? '禁用' : '启用'}
          </Button>
        </Space>
      ),
    },
  ];

  return (
    <div>
      {/* Search bar */}
      <Space style={{ marginBottom: 16 }} wrap>
        <Input
          placeholder="SKU / 名称"
          value={keyword}
          onChange={(e) => setKeyword(e.target.value)}
          onPressEnter={handleSearch}
          style={{ width: 200 }}
          allowClear
        />
        <Select
          value={status}
          onChange={(v) => setStatus(v)}
          options={STATUS_OPTIONS}
          style={{ width: 100 }}
        />
        <Button
          type="primary"
          icon={<SearchOutlined />}
          onClick={handleSearch}
        >
          查询
        </Button>
        <Button
          type="primary"
          icon={<PlusOutlined />}
          onClick={openCreateModal}
        >
          新增商品
        </Button>
      </Space>

      {/* Table */}
      <Table<Product>
        rowKey="id"
        columns={columns}
        dataSource={data?.items ?? []}
        loading={isLoading}
        pagination={{
          current: page,
          pageSize,
          total: data?.total ?? 0,
          showSizeChanger: true,
          showTotal: (total) => `共 ${total} 条`,
        }}
        onChange={handleTableChange}
      />

      {/* Product form modal (create / edit) */}
      <ProductFormModal
        open={modalOpen}
        product={editingProduct}
        onClose={closeModal}
      />
    </div>
  );
}
