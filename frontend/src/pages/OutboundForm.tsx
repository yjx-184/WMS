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
  Tag,
  message,
} from 'antd';
import { PlusOutlined, DeleteOutlined } from '@ant-design/icons';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate, useParams } from 'react-router-dom';
import { listProducts } from '../api/product';
import { listWarehouses } from '../api/warehouse';
import { listLocations } from '../api/location';
import {
  createOutboundOrder,
  getOutboundOrder,
  updateOutboundOrder,
} from '../api/outbound';
import apiClient from '../api/client';
import type { OutboundOrderType } from '../types/outbound';

const ORDER_TYPE_OPTIONS: { value: OutboundOrderType; label: string }[] = [
  { value: 'sales', label: '销售' },
  { value: 'manual', label: '手动' },
  { value: 'scrap', label: '报废' },
];

interface ItemRow {
  key: string;
  product_id?: string;
  location_id?: string;
  planned_qty: number;
  available_qty?: number;
}

let _itemCounter = 0;
function nextKey() {
  _itemCounter += 1;
  return `item-${_itemCounter}`;
}

/** Fetch available stock for a product+warehouse+location combination. */
async function fetchStock(
  productId: string,
  warehouseId: string,
  locationId: string,
): Promise<number | null> {
  try {
    const { data } = await apiClient.get('/inventory', {
      params: {
        product_id: productId,
        warehouse_id: warehouseId,
        location_id: locationId,
        page: 1,
        page_size: 1,
      },
    });
    const item = data.data?.items?.[0];
    return item ? Number(item.quantity) : 0;
  } catch {
    return null; // endpoint not ready
  }
}

export default function OutboundForm() {
  const { id } = useParams<{ id?: string }>();
  const isEdit = Boolean(id);
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [form] = Form.useForm();

  const { data: order } = useQuery({
    queryKey: ['outboundOrder', id],
    queryFn: () => getOutboundOrder(id!),
    enabled: isEdit,
  });

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

  const [items, setItems] = useState<ItemRow[]>([{ key: nextKey(), planned_qty: 1 }]);

  /* ----- guard ----- */
  useEffect(() => {
    if (order && order.status !== 'draft') {
      message.warning('仅草稿状态可编辑');
      navigate('/outbounds', { replace: true });
    }
  }, [order, navigate]);

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

  /* ----- stock lookup when product + location + warehouse selected ----- */
  useEffect(() => {
    items.forEach((it) => {
      if (it.product_id && it.location_id && selectedWarehouse) {
        fetchStock(it.product_id, selectedWarehouse, it.location_id).then(
          (qty) => {
            setItems((prev) =>
              prev.map((r) =>
                r.key === it.key ? { ...r, available_qty: qty ?? undefined } : r,
              ),
            );
          },
        );
      }
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [items.map((i) => `${i.product_id}|${i.location_id}`).join(','), selectedWarehouse]);

  /* ----- mutations ----- */
  const createMutation = useMutation({
    mutationFn: createOutboundOrder,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['outboundOrders'] });
      message.success('保存成功');
      navigate('/outbounds');
    },
  });

  const updateMutation = useMutation({
    mutationFn: (req: Parameters<typeof updateOutboundOrder>[1]) =>
      updateOutboundOrder(id!, req),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['outboundOrders'] });
      message.success('保存成功');
      navigate('/outbounds');
    },
  });

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

  async function handleSave() {
    try {
      const values = await form.validateFields();
      if (items.length === 0) {
        message.error('至少需要一条明细');
        return;
      }
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
      width: 140,
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
      title: '可用库存',
      key: 'stock',
      width: 100,
      render: (_: unknown, r: ItemRow) => {
        if (r.available_qty === undefined) return '-';
        return (
          <Tag color={r.planned_qty > r.available_qty ? 'red' : 'green'}>
            {r.available_qty}
          </Tag>
        );
      },
    },
    {
      title: '计划数量',
      key: 'qty',
      width: 110,
      render: (_: unknown, r: ItemRow) => (
        <InputNumber
          min={1}
          value={r.planned_qty}
          onChange={(v) => updateItem(r.key, { planned_qty: v ?? 1 })}
          style={{
            width: '100%',
            ...(r.available_qty !== undefined && r.planned_qty > r.available_qty
              ? { borderColor: 'red' }
              : {}),
          }}
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
          { title: '出库管理' },
          { title: isEdit ? '编辑出库单' : '新增出库单' },
        ]}
      />

      <Form form={form} layout="vertical" style={{ maxWidth: 880 }}>
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
            label="出库类型"
            rules={[{ required: true }]}
            initialValue="sales"
            style={{ minWidth: 140 }}
          >
            <Select options={ORDER_TYPE_OPTIONS} />
          </Form.Item>
        </Space>

        <Form.Item name="remark" label="备注">
          <Input.TextArea rows={2} placeholder="备注" />
        </Form.Item>
      </Form>

      <div style={{ marginBottom: 8 }}>
        <Space>
          <span style={{ fontWeight: 500 }}>出库明细</span>
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
        style={{ maxWidth: 880, marginBottom: 24 }}
      />

      <Space>
        <Button type="primary" loading={saving} onClick={handleSave}>
          保存草稿
        </Button>
        <Button onClick={() => navigate('/outbounds')}>返回</Button>
      </Space>
    </div>
  );
}
