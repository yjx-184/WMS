import { useState } from 'react';
import {
  Button,
  DatePicker,
  Input,
  Select,
  Space,
  Table,
  Tag,
  type TablePaginationConfig,
} from 'antd';
import { PlusOutlined, SearchOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import { useQuery } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { listWarehouses } from '../api/warehouse';
import { listInboundOrders } from '../api/inbound';
import type {
  InboundOrderListItem,
  InboundOrderListQuery,
  InboundOrderStatus,
  InboundOrderType,
} from '../types/inbound';

const ORDER_TYPE_LABELS: Record<InboundOrderType, string> = {
  purchase: '采购',
  return: '退货',
  manual: '手动',
};

const STATUS_TAGS: Record<InboundOrderStatus, { color: string; label: string }> = {
  draft: { color: 'blue', label: '草稿' },
  completed: { color: 'green', label: '已完成' },
  cancelled: { color: 'default', label: '已取消' },
};

const STATUS_OPTIONS: { value: InboundOrderStatus | ''; label: string }[] = [
  { value: '', label: '全部' },
  { value: 'draft', label: '草稿' },
  { value: 'completed', label: '已完成' },
  { value: 'cancelled', label: '已取消' },
];

export default function InboundList() {
  const navigate = useNavigate();

  /* ----- warehouse options ----- */
  const { data: warehouses } = useQuery({
    queryKey: ['warehouses', { page: 1, page_size: 100 }],
    queryFn: () => listWarehouses({ page: 1, page_size: 100 }),
  });

  /* ----- search state ----- */
  const [keyword, setKeyword] = useState('');
  const [warehouseId, setWarehouseId] = useState<string | undefined>();
  const [status, setStatus] = useState<InboundOrderStatus | ''>('');
  const [dateRange, setDateRange] = useState<[dayjs.Dayjs, dayjs.Dayjs] | null>(null);
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);

  const queryParams: InboundOrderListQuery = {
    keyword: keyword || undefined,
    warehouse_id: warehouseId || undefined,
    status: (status || undefined) as InboundOrderStatus | undefined,
    start_date: dateRange?.[0]?.format('YYYY-MM-DD'),
    end_date: dateRange?.[1]?.format('YYYY-MM-DD'),
    page,
    page_size: pageSize,
  };

  /* ----- data ----- */
  const { data, isLoading } = useQuery({
    queryKey: ['inboundOrders', queryParams],
    queryFn: () => listInboundOrders(queryParams),
  });

  /* ----- search ----- */
  function handleSearch() {
    setPage(1);
  }
  function handleTableChange(p: TablePaginationConfig) {
    if (p.current) setPage(p.current);
    if (p.pageSize) setPageSize(p.pageSize);
  }

  /* ----- columns ----- */
  const columns = [
    { title: '单号', dataIndex: 'order_no', key: 'order_no' },
    {
      title: '仓库',
      dataIndex: 'warehouse_name',
      key: 'warehouse_name',
      render: (v: string) => v || '-',
    },
    {
      title: '类型',
      dataIndex: 'order_type',
      key: 'order_type',
      render: (t: InboundOrderType) => ORDER_TYPE_LABELS[t] ?? t,
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      render: (s: InboundOrderStatus) => {
        const tag = STATUS_TAGS[s];
        return <Tag color={tag.color}>{tag.label}</Tag>;
      },
    },
    {
      title: '创建时间',
      dataIndex: 'created_at',
      key: 'created_at',
      render: (v: string) => v ? dayjs(v).format('YYYY-MM-DD HH:mm') : '-',
    },
    {
      title: '操作',
      key: 'actions',
      render: (_: unknown, r: InboundOrderListItem) => (
        <Space>
          <Button
            type="link"
            size="small"
            onClick={() => navigate(`/inbounds/${r.id}`)}
          >
            {r.status === 'draft' ? '编辑' : '查看'}
          </Button>
          {r.status === 'draft' && (
            <Button
              type="link"
              size="small"
              onClick={() => navigate(`/inbounds/${r.id}`)}
            >
              完成
            </Button>
          )}
        </Space>
      ),
    },
  ];

  return (
    <div>
      <Space style={{ marginBottom: 16 }} wrap>
        <Input
          placeholder="单号"
          value={keyword}
          onChange={(e) => setKeyword(e.target.value)}
          onPressEnter={handleSearch}
          style={{ width: 180 }}
          allowClear
        />
        <Select
          placeholder="仓库"
          value={warehouseId}
          onChange={(v) => setWarehouseId(v)}
          allowClear
          options={(warehouses?.items ?? []).map((w) => ({
            value: w.id,
            label: w.name,
          }))}
          style={{ width: 160 }}
        />
        <Select
          value={status}
          onChange={(v) => setStatus(v)}
          options={STATUS_OPTIONS}
          style={{ width: 110 }}
        />
        <DatePicker.RangePicker
          value={dateRange as any}
          onChange={(v) => setDateRange(v as [dayjs.Dayjs, dayjs.Dayjs] | null)}
          allowClear
        />
        <Button type="primary" icon={<SearchOutlined />} onClick={handleSearch}>
          查询
        </Button>
        <Button
          type="primary"
          icon={<PlusOutlined />}
          onClick={() => navigate('/inbounds/new')}
        >
          新增入库单
        </Button>
      </Space>

      <Table<InboundOrderListItem>
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
