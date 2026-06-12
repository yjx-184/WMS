-- ============================================================
-- WMS MVP — Initial Schema Migration
-- 9 tables, no product_categories, no locked_quantity,
-- no pending/approved status values.
-- ============================================================

-- ------------------------------------------------------------
-- ENUM types
-- ------------------------------------------------------------

CREATE TYPE product_status AS ENUM ('active', 'disabled');

CREATE TYPE warehouse_status AS ENUM ('active', 'disabled');

CREATE TYPE location_type AS ENUM ('normal', 'receiving', 'shipping', 'return');
CREATE TYPE location_status AS ENUM ('active', 'disabled');

CREATE TYPE inbound_order_type AS ENUM ('purchase', 'return', 'manual');
CREATE TYPE inbound_order_status AS ENUM ('draft', 'completed', 'cancelled');

CREATE TYPE outbound_order_type AS ENUM ('sales', 'manual', 'scrap');
CREATE TYPE outbound_order_status AS ENUM ('draft', 'completed', 'cancelled');

CREATE TYPE transaction_change_type AS ENUM ('inbound', 'outbound');

-- ------------------------------------------------------------
-- 4.1 商品
-- ------------------------------------------------------------

CREATE TABLE products (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sku_code    VARCHAR(50) NOT NULL,
    name        VARCHAR(200) NOT NULL,
    unit        VARCHAR(20) NOT NULL DEFAULT 'pcs',
    spec        VARCHAR(500),
    barcode     VARCHAR(100),
    status      product_status NOT NULL DEFAULT 'active',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX uq_products_sku_code ON products(sku_code);
CREATE INDEX idx_products_name ON products(name);
CREATE INDEX idx_products_status ON products(status);

-- ------------------------------------------------------------
-- 4.2 仓库
-- ------------------------------------------------------------

CREATE TABLE warehouses (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code        VARCHAR(50) NOT NULL,
    name        VARCHAR(200) NOT NULL,
    address     VARCHAR(500),
    status      warehouse_status NOT NULL DEFAULT 'active',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX uq_warehouses_code ON warehouses(code);

-- ------------------------------------------------------------
-- 4.3 库位
-- ------------------------------------------------------------

CREATE TABLE locations (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    warehouse_id  UUID NOT NULL REFERENCES warehouses(id),
    code          VARCHAR(50) NOT NULL,
    location_type location_type NOT NULL DEFAULT 'normal',
    max_volume    NUMERIC(10,4),
    max_weight    NUMERIC(10,4),
    status        location_status NOT NULL DEFAULT 'active',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX uq_locations_warehouse_code ON locations(warehouse_id, code);
CREATE INDEX idx_locations_warehouse_id ON locations(warehouse_id);

-- ------------------------------------------------------------
-- 4.4 库存
-- ------------------------------------------------------------

CREATE TABLE inventories (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id    UUID NOT NULL REFERENCES products(id),
    warehouse_id  UUID NOT NULL REFERENCES warehouses(id),
    location_id   UUID NOT NULL REFERENCES locations(id),
    quantity      NUMERIC(18,4) NOT NULL DEFAULT 0
                    CHECK (quantity >= 0),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX uq_inventories_product_location 
    ON inventories(product_id, warehouse_id, location_id);
CREATE INDEX idx_inventories_product_id ON inventories(product_id);
CREATE INDEX idx_inventories_warehouse_id ON inventories(warehouse_id);
CREATE INDEX idx_inventories_location_id ON inventories(location_id);

-- ------------------------------------------------------------
-- 4.5 入库单
-- ------------------------------------------------------------

CREATE TABLE inbound_orders (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_no      VARCHAR(30) NOT NULL,
    warehouse_id  UUID NOT NULL REFERENCES warehouses(id),
    order_type    inbound_order_type NOT NULL DEFAULT 'purchase',
    status        inbound_order_status NOT NULL DEFAULT 'draft',
    remark        VARCHAR(500),
    completed_at  TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX uq_inbound_orders_order_no ON inbound_orders(order_no);
CREATE INDEX idx_inbound_orders_warehouse_id ON inbound_orders(warehouse_id);
CREATE INDEX idx_inbound_orders_status ON inbound_orders(status);
CREATE INDEX idx_inbound_orders_created_at ON inbound_orders(created_at);

CREATE TABLE inbound_order_items (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id        UUID NOT NULL REFERENCES inbound_orders(id) ON DELETE CASCADE,
    product_id      UUID NOT NULL REFERENCES products(id),
    location_id     UUID NOT NULL REFERENCES locations(id),
    planned_qty     NUMERIC(18,4) NOT NULL CHECK (planned_qty > 0),
    actual_qty      NUMERIC(18,4),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_inbound_order_items_order_id ON inbound_order_items(order_id);
CREATE INDEX idx_inbound_order_items_product_id ON inbound_order_items(product_id);

-- ------------------------------------------------------------
-- 4.6 出库单
-- ------------------------------------------------------------

CREATE TABLE outbound_orders (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_no      VARCHAR(30) NOT NULL,
    warehouse_id  UUID NOT NULL REFERENCES warehouses(id),
    order_type    outbound_order_type NOT NULL DEFAULT 'sales',
    status        outbound_order_status NOT NULL DEFAULT 'draft',
    remark        VARCHAR(500),
    completed_at  TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX uq_outbound_orders_order_no ON outbound_orders(order_no);
CREATE INDEX idx_outbound_orders_warehouse_id ON outbound_orders(warehouse_id);
CREATE INDEX idx_outbound_orders_status ON outbound_orders(status);
CREATE INDEX idx_outbound_orders_created_at ON outbound_orders(created_at);

CREATE TABLE outbound_order_items (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id        UUID NOT NULL REFERENCES outbound_orders(id) ON DELETE CASCADE,
    product_id      UUID NOT NULL REFERENCES products(id),
    location_id     UUID NOT NULL REFERENCES locations(id),
    planned_qty     NUMERIC(18,4) NOT NULL CHECK (planned_qty > 0),
    actual_qty      NUMERIC(18,4),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_outbound_order_items_order_id ON outbound_order_items(order_id);
CREATE INDEX idx_outbound_order_items_product_id ON outbound_order_items(product_id);

-- ------------------------------------------------------------
-- 4.7 库存流水
-- ------------------------------------------------------------

CREATE TABLE inventory_transactions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id      UUID NOT NULL REFERENCES products(id),
    warehouse_id    UUID NOT NULL REFERENCES warehouses(id),
    location_id     UUID NOT NULL REFERENCES locations(id),
    change_type     transaction_change_type NOT NULL,
    quantity        NUMERIC(18,4) NOT NULL,
    quantity_before NUMERIC(18,4) NOT NULL,
    quantity_after  NUMERIC(18,4) NOT NULL,
    reference_type  VARCHAR(50) NOT NULL,
    reference_id    UUID NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_inventory_transactions_product_id ON inventory_transactions(product_id);
CREATE INDEX idx_inventory_transactions_warehouse_id ON inventory_transactions(warehouse_id);
CREATE INDEX idx_inventory_transactions_change_type ON inventory_transactions(change_type);
CREATE INDEX idx_inventory_transactions_created_at ON inventory_transactions(created_at);
CREATE INDEX idx_inventory_transactions_reference ON inventory_transactions(reference_type, reference_id);
