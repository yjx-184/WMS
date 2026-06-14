import { useEffect, useState } from 'react';
import {
  Breadcrumb,
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
import { useParams } from 'react-router-dom';
import { getWarehouse } from '../api/warehouse';
import {
  createLocation,
  listLocations,
  updateLocation,
  updateLocationStatus,
} from '../api/location';
import type {
  Location,
  LocationListQuery,
  LocationStatus,
  LocationType,
} from '../types/location';

const TYPE_LABELS: Record<LocationType, string> = {
  normal: '正常',
  receiving: '收货',
  shipping: '发货',
  return: '退货',
};

const STATUS_LABELS: Record<LocationStatus, string> = {
  active: '启用',
  disabled: '禁用',
};

const LOCATION_TYPE_OPTIONS: { value: LocationType | ''; label: string }[] = [
  { value: '', label: '全部' },
  { value: 'normal', label: '正常' },
  { value: 'receiving', label: '收货' },
  { value: 'shipping', label: '发货' },
  { value: 'return', label: '退货' },
];

const STATUS_OPTIONS: { value: LocationStatus | ''; label: string }[] = [
  { value: '', label: '全部' },
  { value: 'active', label: '启用' },
  { value: 'disabled', label: '禁用' },
];

interface FormValues {
  code: string;
  location_type: LocationType;
}

export default function LocationList() {
  const { id: warehouseId } = useParams<{ id: string }>();
  const queryClient = useQueryClient();

  /* ----- warehouse context ----- */
  const { data: warehouse } = useQuery({
    queryKey: ['warehouse', warehouseId],
    queryFn: () => getWarehouse(warehouseId!),
    enabled: !!warehouseId,
  });

  /* ----- modal state ----- */
  const [modalOpen, setModalOpen] = useState(false);
  const [editingLocation, setEditingLocation] = useState<Location | null>(null);
  const [form] = Form.useForm<FormValues>();
  const isEdit = editingLocation !== null;

  useEffect(() => {
    if (modalOpen) {
      if (editingLocation) {
        form.setFieldsValue({
          code: editingLocation.code,
          location_type: editingLocation.location_type,
        });
      } else {
        form.resetFields();
      }
    }
  }, [modalOpen, editingLocation, form]);

  /* ----- search state ----- */
  const [keyword, setKeyword] = useState('');
  const [locationType, setLocationType] = useState<LocationType | ''>('');
  const [status, setStatus] = useState<LocationStatus | ''>('');
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);

  const queryParams: LocationListQuery = {
    keyword: keyword || undefined,
    location_type: (locationType || undefined) as LocationType | undefined,
    status: (status || undefined) as LocationStatus | undefined,
    page,
    page_size: pageSize,
  };

  /* ----- data ----- */
  const { data, isLoading } = useQuery({
    queryKey: ['locations', warehouseId, queryParams],
    queryFn: () => listLocations(warehouseId!, queryParams),
    enabled: !!warehouseId,
  });

  /* ----- mutations ----- */
  const toggleMutation = useMutation({
    mutationFn: ({ id, next }: { id: string; next: LocationStatus }) =>
      updateLocationStatus(id, next),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['locations', warehouseId] });
    },
  });

  const createMutation = useMutation({
    mutationFn: (req: FormValues) => createLocation(warehouseId!, req),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['locations', warehouseId] });
      closeModal();
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, ...req }: { id: string } & FormValues) =>
      updateLocation(id, req),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['locations', warehouseId] });
      closeModal();
    },
  });

  function handleToggle(loc: Location) {
    const next: LocationStatus =
      loc.status === 'active' ? 'disabled' : 'active';
    toggleMutation.mutate({ id: loc.id, next });
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
    setEditingLocation(null);
    setModalOpen(true);
  }

  function openEditModal(loc: Location) {
    setEditingLocation(loc);
    setModalOpen(true);
  }

  function closeModal() {
    setModalOpen(false);
    setEditingLocation(null);
  }

  async function handleModalOk() {
    try {
      const values = await form.validateFields();
      if (isEdit) {
        updateMutation.mutate({ id: editingLocation!.id, ...values });
      } else {
        createMutation.mutate(values);
      }
    } catch {
      // validation failed
    }
  }

  /* ----- columns ----- */
  const columns = [
    { title: '编码', dataIndex: 'code', key: 'code' },
    {
      title: '类型',
      dataIndex: 'location_type',
      key: 'location_type',
      render: (t: LocationType) => TYPE_LABELS[t] ?? t,
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      render: (s: LocationStatus) => (
        <Tag color={s === 'active' ? 'green' : 'red'}>{STATUS_LABELS[s]}</Tag>
      ),
    },
    {
      title: '操作',
      key: 'actions',
      render: (_: unknown, record: Location) => (
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
      {/* Breadcrumb */}
      <Breadcrumb
        style={{ marginBottom: 16 }}
        items={[
          { title: '仓库管理' },
          {
            title: warehouse?.name ?? warehouseId ?? '...',
          },
          { title: '库位管理' },
        ]}
      />

      {/* Search bar */}
      <Space style={{ marginBottom: 16 }} wrap>
        <Input
          placeholder="编码"
          value={keyword}
          onChange={(e) => setKeyword(e.target.value)}
          onPressEnter={handleSearch}
          style={{ width: 160 }}
          allowClear
        />
        <Select
          value={locationType}
          onChange={(v) => setLocationType(v)}
          options={LOCATION_TYPE_OPTIONS}
          style={{ width: 100 }}
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
          新增库位
        </Button>
      </Space>

      {/* Table */}
      <Table<Location>
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
        title={isEdit ? '编辑库位' : '新增库位'}
        open={modalOpen}
        onOk={handleModalOk}
        onCancel={closeModal}
        confirmLoading={
          createMutation.isPending || updateMutation.isPending
        }
        destroyOnClose
        forceRender
      >
        <Form
          form={form}
          layout="vertical"
          initialValues={{ location_type: 'normal' }}
          preserve={false}
        >
          <Form.Item
            name="code"
            label="编码"
            rules={[{ required: true, message: '请输入编码' }]}
          >
            <Input placeholder="请输入编码" />
          </Form.Item>
          <Form.Item
            name="location_type"
            label="类型"
            rules={[{ required: true, message: '请选择类型' }]}
          >
            <Select
              options={[
                { value: 'normal', label: '正常' },
                { value: 'receiving', label: '收货' },
                { value: 'shipping', label: '发货' },
                { value: 'return', label: '退货' },
              ]}
            />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
}
