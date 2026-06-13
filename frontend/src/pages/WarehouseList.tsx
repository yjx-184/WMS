import { useEffect, useState } from 'react';
import {
  Button,
  Form,
  Input,
  Modal,
  Select,
  Space,
  Table,
  Tag,
  type TablePaginationConfig,
} from 'antd';
import { PlusOutlined, SearchOutlined } from '@ant-design/icons';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import {
  createWarehouse,
  listWarehouses,
  updateWarehouse,
  updateWarehouseStatus,
} from '../api/warehouse';
import type {
  Warehouse,
  WarehouseListQuery,
  WarehouseStatus,
} from '../types/warehouse';

const STATUS_LABELS: Record<WarehouseStatus, string> = {
  active: '启用',
  disabled: '禁用',
};

const STATUS_OPTIONS: { value: WarehouseStatus | ''; label: string }[] = [
  { value: '', label: '全部' },
  { value: 'active', label: '启用' },
  { value: 'disabled', label: '禁用' },
];

interface FormValues {
  code: string;
  name: string;
  address?: string;
}

export default function WarehouseList() {
  const queryClient = useQueryClient();
  const navigate = useNavigate();

  /* ----- modal state ----- */
  const [modalOpen, setModalOpen] = useState(false);
  const [editingWarehouse, setEditingWarehouse] = useState<Warehouse | null>(
    null,
  );
  const [form] = Form.useForm<FormValues>();
  const isEdit = editingWarehouse !== null;

  useEffect(() => {
    if (modalOpen) {
      if (editingWarehouse) {
        form.setFieldsValue({
          code: editingWarehouse.code,
          name: editingWarehouse.name,
          address: editingWarehouse.address ?? undefined,
        });
      } else {
        form.resetFields();
      }
    }
  }, [modalOpen, editingWarehouse, form]);

  /* ----- search state ----- */
  const [keyword, setKeyword] = useState('');
  const [status, setStatus] = useState<WarehouseStatus | ''>('');
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);

  const queryParams: WarehouseListQuery = {
    keyword: keyword || undefined,
    status: (status || undefined) as WarehouseStatus | undefined,
    page,
    page_size: pageSize,
  };

  /* ----- data ----- */
  const { data, isLoading } = useQuery({
    queryKey: ['warehouses', queryParams],
    queryFn: () => listWarehouses(queryParams),
  });

  /* ----- mutations ----- */
  const toggleMutation = useMutation({
    mutationFn: ({ id, next }: { id: string; next: WarehouseStatus }) =>
      updateWarehouseStatus(id, next),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['warehouses'] });
    },
  });

  const createMutation = useMutation({
    mutationFn: createWarehouse,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['warehouses'] });
      closeModal();
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, ...req }: { id: string } & FormValues) =>
      updateWarehouse(id, req),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['warehouses'] });
      closeModal();
    },
  });

  function handleToggle(wh: Warehouse) {
    const next: WarehouseStatus = wh.status === 'active' ? 'disabled' : 'active';
    toggleMutation.mutate({ id: wh.id, next });
  }

  /* ----- search ----- */
  function handleSearch() {
    setPage(1);
  }

  function handleTableChange(pagination: TablePaginationConfig) {
    if (pagination.current) setPage(pagination.current);
    if (pagination.pageSize) setPageSize(pagination.pageSize);
  }

  /* ----- modal ----- */
  function openCreateModal() {
    setEditingWarehouse(null);
    setModalOpen(true);
  }

  function openEditModal(wh: Warehouse) {
    setEditingWarehouse(wh);
    setModalOpen(true);
  }

  function closeModal() {
    setModalOpen(false);
    setEditingWarehouse(null);
  }

  async function handleModalOk() {
    try {
      const values = await form.validateFields();
      if (isEdit) {
        updateMutation.mutate({ id: editingWarehouse!.id, ...values });
      } else {
        createMutation.mutate(values);
      }
    } catch {
      // validation failed — Ant Form shows inline errors
    }
  }

  /* ----- columns ----- */
  const columns = [
    { title: '编码', dataIndex: 'code', key: 'code' },
    { title: '名称', dataIndex: 'name', key: 'name' },
    {
      title: '地址',
      dataIndex: 'address',
      key: 'address',
      render: (v: string | null) => v ?? '-',
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      render: (s: WarehouseStatus) => (
        <Tag color={s === 'active' ? 'green' : 'red'}>{STATUS_LABELS[s]}</Tag>
      ),
    },
    {
      title: '操作',
      key: 'actions',
      render: (_: unknown, record: Warehouse) => (
        <Space>
          <Button type="link" size="small" onClick={() => openEditModal(record)}>
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
          <Button
            type="link"
            size="small"
            onClick={() => navigate(`/warehouses/${record.id}/locations`)}
          >
            库位
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
          placeholder="名称 / 编码"
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
        <Button type="primary" icon={<SearchOutlined />} onClick={handleSearch}>
          查询
        </Button>
        <Button type="primary" icon={<PlusOutlined />} onClick={openCreateModal}>
          新增仓库
        </Button>
      </Space>

      {/* Table */}
      <Table<Warehouse>
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

      {/* Create / Edit Modal */}
      <Modal
        title={isEdit ? '编辑仓库' : '新增仓库'}
        open={modalOpen}
        onOk={handleModalOk}
        onCancel={closeModal}
        confirmLoading={createMutation.isPending || updateMutation.isPending}
        destroyOnClose
        forceRender
      >
        <Form form={form} layout="vertical" preserve={false}>
          <Form.Item
            name="code"
            label="编码"
            rules={[{ required: true, message: '请输入编码' }]}
          >
            <Input placeholder="请输入编码" />
          </Form.Item>
          <Form.Item
            name="name"
            label="名称"
            rules={[{ required: true, message: '请输入名称' }]}
          >
            <Input placeholder="请输入名称" />
          </Form.Item>
          <Form.Item name="address" label="地址">
            <Input placeholder="请输入地址" />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
}
