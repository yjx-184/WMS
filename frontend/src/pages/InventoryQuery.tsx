import { useState } from 'react';
import { Input, Select, Space, Table, type TablePaginationConfig } from 'antd';
import { SearchOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import { useQuery } from '@tanstack/react-query';
import { listProducts } from '../api/product';
import { listWarehouses } from '../api/warehouse';
import { listLocations } from '../api/location';
import { queryInventory } from '../api/inventory';
import type { InventoryQueryParams, InventoryRow } from '../types/inventory';

export default function InventoryQuery() {
  const [productId, setProductId] = useState<string | undefined>();
  const [warehouseId, setWarehouseId] = useState<string | undefined>();
  const [locationId, setLocationId] = useState<string | undefined>();
  const [keyword, setKeyword] = useState('');
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);

  const { data: products } = useQuery({
    queryKey: ['products', { page: 1, page_size: 200 }],
    queryFn: () => listProducts({ page: 1, page_size: 200 }),
  });
  const { data: warehouses } = useQuery({
    queryKey: ['warehouses', { page: 1, page_size: 100 }],
    queryFn: () => listWarehouses({ page: 1, page_size: 100 }),
  });
  const { data: locations } = useQuery({
    queryKey: ['locations', warehouseId, { page: 1, page_size: 200 }],
    queryFn: () => listLocations(warehouseId!, { page: 1, page_size: 200 }),
    enabled: Boolean(warehouseId),
  });

  const params: InventoryQueryParams = {
    product_id: productId || undefined,
    warehouse_id: warehouseId || undefined,
    location_id: locationId || undefined,
    keyword: keyword || undefined,
    page,
    page_size: pageSize,
  };

  const { data, isLoading } = useQuery({
    queryKey: ['inventory', params],
    queryFn: () => queryInventory(params),
  });

  function handleSearch() {
    setPage(1);
  }
  function handleTableChange(p: TablePaginationConfig) {
    if (p.current) setPage(p.current);
    if (p.pageSize) setPageSize(p.pageSize);
  }

  const columns = [
    { title: '商品名称', dataIndex: 'product_name', key: 'product_name' },
    { title: 'SKU', dataIndex: 'sku_code', key: 'sku_code' },
    { title: '仓库', dataIndex: 'warehouse_name', key: 'warehouse_name' },
    { title: '库位', dataIndex: 'location_code', key: 'location_code' },
    { title: '库存数量', dataIndex: 'quantity', key: 'quantity' },
    {
      title: '更新时间',
      dataIndex: 'updated_at',
      key: 'updated_at',
      render: (v: string) => (v ? dayjs(v).format('YYYY-MM-DD HH:mm') : '-'),
    },
  ];

  return (
    <div>
      <Space style={{ marginBottom: 16 }} wrap>
        <Select
          showSearch
          placeholder="商品"
          value={productId}
          onChange={(v) => setProductId(v)}
          allowClear
          options={(products?.items ?? []).map((p) => ({
            value: p.id,
            label: `${p.sku_code} ${p.name}`,
          }))}
          filterOption={(input, option) =>
            (option?.label as string)?.toLowerCase().includes(input.toLowerCase())
          }
          style={{ width: 200 }}
        />
        <Select
          placeholder="仓库"
          value={warehouseId}
          onChange={(v) => {
            setWarehouseId(v);
            setLocationId(undefined);
          }}
          allowClear
          options={(warehouses?.items ?? []).map((w) => ({
            value: w.id,
            label: w.name,
          }))}
          style={{ width: 160 }}
        />
        <Select
          placeholder="库位"
          value={locationId}
          onChange={(v) => setLocationId(v)}
          allowClear
          disabled={!warehouseId}
          options={(locations?.items ?? []).map((l) => ({
            value: l.id,
            label: l.code,
          }))}
          style={{ width: 140 }}
        />
        <Input
          placeholder="关键字"
          value={keyword}
          onChange={(e) => setKeyword(e.target.value)}
          onPressEnter={handleSearch}
          style={{ width: 160 }}
          allowClear
        />
        <Space.Compact>
          <SearchOutlined
            style={{ padding: '8px', cursor: 'pointer', fontSize: 16 }}
            onClick={handleSearch}
          />
        </Space.Compact>
      </Space>

      <Table<InventoryRow>
        rowKey="id"
        columns={columns}
        dataSource={data?.items ?? []}
        loading={isLoading}
        pagination={{
          current: page,
          pageSize,
          total: data?.total ?? 0,
          showSizeChanger: true,
          showTotal: (t) => `共 ${t} 条`,
        }}
        onChange={handleTableChange}
      />
    </div>
  );
}
