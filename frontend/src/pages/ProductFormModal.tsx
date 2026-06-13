import { useEffect, useRef } from 'react';
import { Form, Input, Modal, Select } from 'antd';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { checkSku, createProduct, updateProduct } from '../api/product';
import type { Product } from '../types/product';

interface ProductFormModalProps {
  open: boolean;
  /** The product being edited, or null when creating. */
  product: Product | null;
  onClose: () => void;
}

interface FormValues {
  sku_code: string;
  name: string;
  unit: string;
  spec?: string;
  barcode?: string;
}

export default function ProductFormModal({
  open,
  product,
  onClose,
}: ProductFormModalProps) {
  const [form] = Form.useForm<FormValues>();
  const queryClient = useQueryClient();
  const isEdit = product !== null;

  /* Reset form when modal opens / product changes */
  useEffect(() => {
    if (open) {
      if (product) {
        form.setFieldsValue({
          sku_code: product.sku_code,
          name: product.name,
          unit: product.unit,
          spec: product.spec ?? undefined,
          barcode: product.barcode ?? undefined,
        });
      } else {
        form.resetFields();
      }
    }
  }, [open, product, form]);

  /* ----- SKU uniqueness check (debounced) ----- */
  const debounceRef = useRef<ReturnType<typeof setTimeout>>();

  async function handleSkuBlur() {
    const sku = form.getFieldValue('sku_code');
    if (!sku) return;

    try {
      const available = await checkSku(sku, product?.id);
      if (!available) {
        form.setFields([
          { name: 'sku_code', errors: ['SKU 编码已存在'] },
        ]);
      }
    } catch {
      // API errors are handled by the Axios interceptor (toast)
    }
  }

  function handleSkuChange() {
    // Clear any previous SKU error while the user is still typing
    if (debounceRef.current) clearTimeout(debounceRef.current);
    form.setFields([{ name: 'sku_code', errors: [] }]);
  }

  /* ----- submit ----- */
  const createMutation = useMutation({
    mutationFn: createProduct,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['products'] });
      onClose();
    },
  });

  const updateMutation = useMutation({
    mutationFn: (req: { id: string } & FormValues) =>
      updateProduct(req.id, req),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['products'] });
      onClose();
    },
  });

  async function handleOk() {
    try {
      const values = await form.validateFields();
      if (isEdit) {
        updateMutation.mutate({ id: product!.id, ...values });
      } else {
        createMutation.mutate(values);
      }
    } catch {
      // validation failed — Ant Form shows inline errors
    }
  }

  function handleCancel() {
    onClose();
  }

  const isSaving = createMutation.isPending || updateMutation.isPending;

  return (
    <Modal
      title={isEdit ? '编辑商品' : '新增商品'}
      open={open}
      onOk={handleOk}
      onCancel={handleCancel}
      confirmLoading={isSaving}
      destroyOnClose
      forceRender
    >
      <Form
        form={form}
        layout="vertical"
        initialValues={{ unit: 'pcs' }}
        preserve={false}
      >
        <Form.Item
          name="sku_code"
          label="SKU 编码"
          rules={[{ required: true, message: '请输入 SKU 编码' }]}
        >
          <Input
            placeholder="请输入 SKU 编码"
            onBlur={handleSkuBlur}
            onChange={handleSkuChange}
          />
        </Form.Item>

        <Form.Item
          name="name"
          label="名称"
          rules={[{ required: true, message: '请输入名称' }]}
        >
          <Input placeholder="请输入名称" />
        </Form.Item>

        <Form.Item
          name="unit"
          label="单位"
          rules={[{ required: true, message: '请选择单位' }]}
        >
          <Select
            options={[
              { value: 'pcs', label: 'pcs' },
              { value: 'box', label: 'box' },
              { value: 'kg', label: 'kg' },
            ]}
          />
        </Form.Item>

        <Form.Item name="spec" label="规格">
          <Input placeholder="请输入规格" />
        </Form.Item>

        <Form.Item name="barcode" label="条形码">
          <Input placeholder="请输入条形码" />
        </Form.Item>
      </Form>
    </Modal>
  );
}
