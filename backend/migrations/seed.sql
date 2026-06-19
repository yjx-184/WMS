-- ============================================================
-- WMS MVP — Seed Data
-- Idempotent: uses INSERT ... ON CONFLICT DO NOTHING.
-- 5 products + 1 warehouse + 4 locations.  No category data.
-- ============================================================

-- Products (5)
INSERT INTO products (sku_code, name, unit, spec)
VALUES ('SKU-0001', 'iPhone 15',           'pcs', '128GB 黑色')
ON CONFLICT (sku_code) DO NOTHING;

INSERT INTO products (sku_code, name, unit, spec, barcode)
VALUES ('SKU-0002', 'AirPods Pro',         'pcs', '第二代', '6901234567890')
ON CONFLICT (sku_code) DO NOTHING;

INSERT INTO products (sku_code, name, unit, spec)
VALUES ('SKU-0003', 'MacBook Pro 14',      'pcs', 'M3 Pro 18GB/512GB')
ON CONFLICT (sku_code) DO NOTHING;

INSERT INTO products (sku_code, name, unit)
VALUES ('SKU-0004', 'USB-C 数据线',        'pcs')
ON CONFLICT (sku_code) DO NOTHING;

INSERT INTO products (sku_code, name, unit, spec)
VALUES ('SKU-0005', '包装纸箱 600×400×300','pcs', '5层瓦楞')
ON CONFLICT (sku_code) DO NOTHING;

-- Warehouse (1)
INSERT INTO warehouses (code, name, address)
VALUES ('WH-MAIN', '主仓库', '默认主仓库')
ON CONFLICT (code) DO NOTHING;

-- Locations (4) — linked to the warehouse above by code
INSERT INTO locations (warehouse_id, code, location_type)
SELECT w.id, 'A-01-01', 'normal'
FROM warehouses w
WHERE w.code = 'WH-MAIN'
ON CONFLICT (warehouse_id, code) DO NOTHING;

INSERT INTO locations (warehouse_id, code, location_type)
SELECT w.id, 'A-01-02', 'normal'
FROM warehouses w
WHERE w.code = 'WH-MAIN'
ON CONFLICT (warehouse_id, code) DO NOTHING;

INSERT INTO locations (warehouse_id, code, location_type)
SELECT w.id, 'RECV-01', 'receiving'
FROM warehouses w
WHERE w.code = 'WH-MAIN'
ON CONFLICT (warehouse_id, code) DO NOTHING;

INSERT INTO locations (warehouse_id, code, location_type)
SELECT w.id, 'SHIP-01', 'shipping'
FROM warehouses w
WHERE w.code = 'WH-MAIN'
ON CONFLICT (warehouse_id, code) DO NOTHING;
