import { useState } from 'react';
import {
  Breadcrumb,
  Button,
  Descriptions,
  InputNumber,
  Modal,
  Space,
  Table,
  Tag,
  message,
} from 'antd';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate, useParams } from 'react-router-dom';
import dayjs from 'dayjs';
import { getInboundOrder, completeInboundOrder, cancelInboundOrder } from '../api/inbound';
import type {
  InboundOrderStatus,
  InboundOrderType,
  InboundOrderItemDetail,
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

export default function InboundDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const { data: order, isLoading } = useQuery({
    queryKey: ['inboundOrder', id],
    queryFn: () => getInboundOrder(id!),
    enabled: Boolean(id),
  });

  /* ----- complete modal state ----- */
  const [modalOpen, setModalOpen] = useState(false);
  const [actualQtys, setActualQtys] = useState<Record<string, number>>({});

  const completeMutation = useMutation({
    mutationFn: (req: { items: { item_id: string; actual_qty: number }[] }) =>
      completeInboundOrder(id!, req),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['inboundOrders'] });
      queryClient.invalidateQueries({ queryKey: ['inboundOrder', id] });
      message.success('入库完成');
      setModalOpen(false);
    },
  });

  const cancelMutation = useMutation({
    mutationFn: () => cancelInboundOrder(id!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['inboundOrders'] });
      queryClient.invalidateQueries({ queryKey: ['inboundOrder', id] });
      message.success('已取消');
    },
  });

  function handleCancel() {
    const isCompleted = order?.status === 'completed';
    Modal.confirm({
      title: '确认取消',
      content: isCompleted
        ? '确认取消该入库单？该操作将回滚库存。'
        : '确认取消该入库单？',
      okText: '确认取消',
      cancelText: '保留',
      okButtonProps: { danger: true },
      onOk: () => cancelMutation.mutate(),
    });
  }

  function openCompleteModal() {
    if (!order) return;
    const init: Record<string, number> = {};
    order.items.forEach((it) => {
      init[it.id] = Number(it.planned_qty);
    });
    setActualQtys(init);
    setModalOpen(true);
  }

  function handleCompleteOk() {
    const items = order!.items.map((it) => ({
      item_id: it.id,
      actual_qty: actualQtys[it.id] ?? Number(it.planned_qty),
    }));
    completeMutation.mutate({ items });
  }

  if (isLoading) return null;
  if (!order) return null;

  const statusTag = STATUS_TAGS[order.status];
  const isDraft = order.status === 'draft';
  const isCancelled = order.status === 'cancelled';

  const itemColumns = [
    { title: '商品名称', dataIndex: 'product_name', key: 'product_name' },
    { title: 'SKU', dataIndex: 'sku_code', key: 'sku_code' },
    { title: '库位', dataIndex: 'location_code', key: 'location_code' },
    { title: '计划数量', dataIndex: 'planned_qty', key: 'planned_qty' },
    {
      title: '实收数量',
      dataIndex: 'actual_qty',
      key: 'actual_qty',
      render: (v: string | null) => v ?? '-',
    },
  ];

  return (
    <div>
      <Breadcrumb
        style={{ marginBottom: 16 }}
        items={[
          { title: '入库管理' },
          { title: `入库单 ${order.order_no}` },
        ]}
      />

      <Descriptions bordered column={2} size="small" style={{ marginBottom: 24 }}>
        <Descriptions.Item label="单号">{order.order_no}</Descriptions.Item>
        <Descriptions.Item label="状态">
          <Tag color={statusTag.color}>{statusTag.label}</Tag>
        </Descriptions.Item>
        <Descriptions.Item label="仓库">{order.warehouse_name}</Descriptions.Item>
        <Descriptions.Item label="类型">
          {ORDER_TYPE_LABELS[order.order_type]}
        </Descriptions.Item>
        <Descriptions.Item label="备注" span={2}>
          {order.remark || '-'}
        </Descriptions.Item>
        <Descriptions.Item label="创建时间">
          {dayjs(order.created_at).format('YYYY-MM-DD HH:mm')}
        </Descriptions.Item>
        <Descriptions.Item label="完成时间">
          {order.completed_at
            ? dayjs(order.completed_at).format('YYYY-MM-DD HH:mm')
            : '-'}
        </Descriptions.Item>
      </Descriptions>

      <h4>入库明细</h4>
      <Table<InboundOrderItemDetail>
        rowKey="id"
        columns={itemColumns}
        dataSource={order.items}
        pagination={false}
        size="small"
        style={{ marginBottom: 24 }}
      />

      <Space>
        <Button onClick={() => navigate('/inbounds')}>返回</Button>
        {isDraft && (
          <Button type="primary" onClick={openCompleteModal}>
            完成入库
          </Button>
        )}
        {!isCancelled && (
          <Button danger onClick={handleCancel}>
            取消
          </Button>
        )}
      </Space>

      <Modal
        title="完成入库"
        open={modalOpen}
        onOk={handleCompleteOk}
        onCancel={() => setModalOpen(false)}
        confirmLoading={completeMutation.isPending}
        destroyOnClose
      >
        <p>请确认每行明细的实收数量：</p>
        {order.items.map((it) => (
          <div key={it.id} style={{ marginBottom: 12 }}>
            <span>
              {it.product_name} ({it.sku_code}) — 计划: {it.planned_qty}
            </span>
            <InputNumber
              min={0}
              value={actualQtys[it.id]}
              onChange={(v) =>
                setActualQtys((prev) => ({ ...prev, [it.id]: v ?? 0 }))
              }
              style={{ marginLeft: 12, width: 100 }}
            />
          </div>
        ))}
      </Modal>
    </div>
  );
}
