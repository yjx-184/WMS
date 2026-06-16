import { useEffect, useState } from 'react';
import {
  Breadcrumb,
  Button,
  Form,
  Input,
  InputNumber,
  Select,
  Space,
  Table,
  message,
} from 'antd';
import { PlusOutlined, DeleteOutlined } from '@ant-design/icons';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate, useParams } from 'react-router-dom';
import { listProducts } from '../api/product';
import { listWarehouses } from '../api/warehouse';
import { listLocations } from '../api/location';
import {
  createInboundOrder,
  getInboundOrder,
  updateInboundOrder,
} from '../api/inbound';
import type { InboundOrderType } from '../types/inbound';

const ORDER_TYPE_OPTIONS: { value: InboundOrderType; label: string }[] = [
  { value: 'purchase', label: '采购' },
  { value: 'return', label: '退货' },
  { value: 'manual', label: '手动' },
];

interface ItemRow {
  key: string;
  product_id?: string;
  location_id?: string;
  planned_qty: number;
}

let _itemCounter = 0;
function nextKey() {
  _itemCounter += 1;
  return `item-${_itemCounter}`;
}

export default function InboundForm() {
  const { id } = useParams<{ id?: string }>();
  const isEdit = Boolean(id);
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [form] = Form.useForm();

  /* ----- loaded order (edit mode) ----- */
  const { data: order } = useQuery({
    queryKey: ['inboundOrder', id],
    queryFn: () => getInboundOrder(id!),
    enabled: isEdit,
  });

  /* ----- dropdown data ----- */
  const { data: warehouses } = useQuery({
    queryKey: ['warehouses', { page: 1, page_size: 100 }],
    queryFn: () => listWarehouses({ page: 1, page_size: 100 }),
  });

  const { data: products } = useQuery({
    queryKey: ['products', { page: 1, page_size: 200 }],
    queryFn: () => listProducts({ page: 1, page_size: 200 }),
  });

  const [selectedWarehouse, setSelectedWarehouse] = useState<string | undefined>();

  const { data: locations } = useQuery({
    queryKey: ['locations', selectedWarehouse, { page: 1, page_size: 200 }],
    queryFn: () => listLocations(selectedWarehouse!, { page: 1, page_size: 200 }),
    enabled: Boolean(selectedWarehouse),
  });

  /* ----- items state ----- */
  const [items, setItems] = useState<ItemRow[]>([{ key: nextKey(), planned_qty: 1 }]);

  /* ----- guard: non-draft orders cannot be edited ----- */
  useEffect(() => {
    if (order && order.status !== 'draft') {
      message.warning('仅草稿状态可编辑');
      navigate('/inbounds', { replace: true });
    }
  }, [order, navigate]);

  /* ----- fill form on edit ----- */
  useEffect(() => {
    if (order && order.status === 'draft') {
      form.setFieldsValue({
        warehouse_id: order.warehouse_id,
        order_type: order.order_type,
        remark: order.remark ?? '',
      });
      setSelectedWarehouse(order.warehouse_id);
      setItems(
        order.items.map((it) => ({
          key: nextKey(),
          product_id: it.product_id,
          location_id: it.location_id,
          planned_qty: Number(it.planned_qty),
        })),
      );
    }
  }, [order, form]);

  /* ----- mutations ----- */
  const createMutation = useMutation({
    mutationFn: createInboundOrder,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['inboundOrders'] });
      message.success('保存成功');
      navigate('/inbounds');
    },
  });

  const updateMutation = useMutation({
    mutationFn: (req: Parameters<typeof updateInboundOrder>[1]) =>
      updateInboundOrder(id!, req),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['inboundOrders'] });
      message.success('保存成功');
      navigate('/inbounds');
    },
  });

  /* ----- item helpers ----- */
  function addItem() {
    setItems((prev) => [...prev, { key: nextKey(), planned_qty: 1 }]);
  }

  function removeItem(key: string) {
    setItems((prev) => prev.filter((r) => r.key !== key));
  }

  function updateItem(key: string, patch: Partial<ItemRow>) {
    setItems((prev) =>
      prev.map((r) => (r.key === key ? { ...r, ...patch } : r)),
    );
  }

  /* ----- submit ----- */
  async function handleSave() {
    try {
      const values = await form.validateFields();
      if (items.length === 0) {
        message.error('至少需要一条明细');
        return;
      }
      // Validate every item row
      for (const it of items) {
        if (!it.product_id) {
          message.error('每条明细必须选择商品');
          return;
        }
        if (!it.location_id) {
          message.error('每条明细必须选择库位');
          return;
        }
        if (!it.planned_qty || it.planned_qty <= 0) {
          message.error('计划数量必须大于0');
          return;
        }
      }
      const req = {
        warehouse_id: values.warehouse_id,
        order_type: values.order_type,
        remark: values.remark || undefined,
        items: items.map((it) => ({
          product_id: it.product_id!,
          location_id: it.location_id!,
          planned_qty: it.planned_qty,
        })),
      };
      if (isEdit) {
        updateMutation.mutate(req);
      } else {
        createMutation.mutate(req);
      }
    } catch {
      // validation failed
    }
  }

  const saving = createMutation.isPending || updateMutation.isPending;

  /* ----- item columns ----- */
  const itemColumns = [
    {
      title: '商品',
      key: 'product',
      width: 200,
      render: (_: unknown, r: ItemRow) => (
        <Select
          showSearch
          placeholder="选择商品"
          value={r.product_id}
          onChange={(v) => updateItem(r.key, { product_id: v })}
          options={(products?.items ?? []).map((p) => ({
            value: p.id,
            label: `${p.sku_code} ${p.name}`,
          }))}
          filterOption={(input, option) =>
            (option?.label as string)?.toLowerCase().includes(input.toLowerCase())
          }
          style={{ width: '100%' }}
        />
      ),
    },
    {
      title: '库位',
      key: 'location',
      width: 160,
      render: (_: unknown, r: ItemRow) => (
        <Select
          placeholder="选择库位"
          value={r.location_id}
          onChange={(v) => updateItem(r.key, { location_id: v })}
          options={(locations?.items ?? []).map((l) => ({
            value: l.id,
            label: l.code,
          }))}
          disabled={!selectedWarehouse}
          style={{ width: '100%' }}
        />
      ),
    },
    {
      title: '计划数量',
      key: 'qty',
      width: 120,
      render: (_: unknown, r: ItemRow) => (
        <InputNumber
          min={1}
          value={r.planned_qty}
          onChange={(v) => updateItem(r.key, { planned_qty: v ?? 1 })}
          style={{ width: '100%' }}
        />
      ),
    },
    {
      title: '',
      key: 'actions',
      width: 40,
      render: (_: unknown, r: ItemRow) => (
        <Button
          type="text"
          danger
          icon={<DeleteOutlined />}
          onClick={() => removeItem(r.key)}
        />
      ),
    },
  ];

  return (
    <div>
      <Breadcrumb
        style={{ marginBottom: 16 }}
        items={[
          { title: '入库管理' },
          { title: isEdit ? '编辑入库单' : '新增入库单' },
        ]}
      />

      <Form form={form} layout="vertical" style={{ maxWidth: 800 }}>
        <Space wrap size="large">
          <Form.Item
            name="warehouse_id"
            label="目标仓库"
            rules={[{ required: true, message: '请选择仓库' }]}
            style={{ minWidth: 200 }}
          >
            <Select
              placeholder="选择仓库"
              onChange={(v) => setSelectedWarehouse(v)}
              options={(warehouses?.items ?? []).map((w) => ({
                value: w.id,
                label: w.name,
              }))}
            />
          </Form.Item>

          <Form.Item
            name="order_type"
            label="入库类型"
            rules={[{ required: true }]}
            initialValue="purchase"
            style={{ minWidth: 140 }}
          >
            <Select options={ORDER_TYPE_OPTIONS} />
          </Form.Item>
        </Space>

        <Form.Item name="remark" label="备注">
          <Input.TextArea rows={2} placeholder="备注" />
        </Form.Item>
      </Form>

      {/* items */}
      <div style={{ marginBottom: 8 }}>
        <Space>
          <span style={{ fontWeight: 500 }}>入库明细</span>
          <Button type="dashed" icon={<PlusOutlined />} onClick={addItem}>
            添加明细
          </Button>
        </Space>
      </div>

      <Table<ItemRow>
        rowKey="key"
        columns={itemColumns}
        dataSource={items}
        pagination={false}
        size="small"
        style={{ maxWidth: 800, marginBottom: 24 }}
      />

      <Space>
        <Button type="primary" loading={saving} onClick={handleSave}>
          保存草稿
        </Button>
        <Button onClick={() => navigate('/inbounds')}>返回</Button>
      </Space>
    </div>
  );
}
