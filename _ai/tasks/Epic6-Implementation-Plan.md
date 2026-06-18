# Epic 6：库存查询 — 实施方案

> 版本：v1.0
> 角色：Tech Lead / Codex
> 目标读者：Reasonix / Codex Reviewer / GPT 架构讨论 / 后续交接
> 预计总工时：4.5h
> 依据：`_ai/tasks/TASK_BOARD.md`、`_ai/PROJECT_HANDOFF_CONTEXT_V4.md`、`_ai/openspec/specs/003-database-design.md`、`_ai/openspec/specs/004-api-design.md`、`_ai/openspec/specs/005-ui-design.md`

---

## 1. Epic Overview

Epic 6 的目标是完成库存查询能力，使 WMS MVP 可以按商品、仓库、库位查询当前库存，并保留库存流水查询 API 供后端测试和审计使用。

Epic 6 完成后，系统将获得：

- 当前库存分页查询能力
- 按商品、仓库、库位、关键字过滤库存的能力
- 库存查询结果展示商品名称、SKU、仓库名称、库位编码、库存数量、更新时间
- 库存流水分页查询能力，仅供后端或测试使用
- 前端只读库存查询页面

Epic 6 严格对应 `TASK_BOARD.md` 中三个任务：

- T6.1.1 实现 StockQueryService
- T6.1.2 实现 Inventory Handler 并注册路由
- T6.2.1 实现库存查询页面

Epic 6 不新增数据库表，不修改数据库 schema，不实现库存锁定，不实现审批流，不实现库存流水前端页面。

---

## 2. Scope Boundaries

### 必须遵守

- 库存粒度固定为 `product_id + warehouse_id + location_id`
- `available` 在后端内部如需表达，固定等于 `quantity`
- API 响应不返回 `locked_quantity`
- API 响应不返回 `available_quantity`
- 库存流水 `change_type` 仅允许 `inbound` / `outbound`
- 库存流水查询 API 仅供后端、测试、审计使用
- 前端 `/inventory` 页面只读

### 严禁实现

- 新增表
- 修改迁移文件
- 恢复 `product_categories`
- 恢复 `products.category_id`
- 恢复 `inventories.locked_quantity`
- 引入 `pending` / `approved`
- 新增 submit / approve / lock / unlock 端点
- 实现库存锁定或解锁逻辑
- 实现库存流水前端页面
- 修改入库、出库状态机

---

## 3. Backend Design

### 3.1 StockQueryService

新增文件：

```text
backend/src/service/stock_query_service.rs
```

并在：

```text
backend/src/service/mod.rs
```

注册模块。

`StockQueryService` 只负责查询编排，不负责库存增减，不负责写入流水。

预期方法：

```text
query_inventory
query_transactions
```

`query_inventory` 能力：

- 分页查询 `inventories`
- 支持 `product_id`
- 支持 `warehouse_id`
- 支持 `location_id`
- 支持 `keyword`
- JOIN `products` 获取 `product_name` / `sku_code`
- JOIN `warehouses` 获取 `warehouse_name`
- JOIN `locations` 获取 `location_code`
- 返回 `quantity`
- 返回 `updated_at`
- 查询结果按 `updated_at DESC` 排序

`keyword` 过滤范围：

- `products.name`
- `products.sku_code`
- `warehouses.name`
- `locations.code`

`query_transactions` 能力：

- 分页查询 `inventory_transactions`
- 支持 `product_id`
- 支持 `warehouse_id`
- 支持 `location_id`
- 支持 `change_type`
- 支持 `start_date`
- 支持 `end_date`
- JOIN `products` / `warehouses` / `locations` 获取展示字段
- `change_type` 仅允许 `inbound` / `outbound`
- 查询结果按 `created_at DESC` 排序

### 3.2 DTO

Epic 6 可新增库存查询 DTO 文件：

```text
backend/src/dto/inventory.rs
```

并在：

```text
backend/src/dto/mod.rs
```

注册模块。

建议 DTO：

- `InventoryQuery`
- `InventoryListItem`
- `InventoryTransactionQuery`
- `InventoryTransactionListItem`

分页响应应复用项目现有列表响应形态：

```json
{
  "items": [],
  "total": 0,
  "page": 1,
  "page_size": 20
}
```

### 3.3 Handler and Routes

仅 T6.1.2 可新增 Handler 与真实路由。

新增文件：

```text
backend/src/handler/inventory_handler.rs
```

并在：

```text
backend/src/handler/mod.rs
backend/src/router.rs
```

注册。

允许实现的端点仅有：

```text
GET /api/v1/inventory
GET /api/v1/inventory-transactions
```

`GET /api/v1/inventory` 查询参数：

- `product_id`
- `warehouse_id`
- `location_id`
- `keyword`
- `page`
- `page_size`

`GET /api/v1/inventory-transactions` 查询参数：

- `product_id`
- `warehouse_id`
- `location_id`
- `change_type`
- `start_date`
- `end_date`
- `page`
- `page_size`

---

## 4. Frontend Design

仅 T6.2.1 可实现前端页面。

允许新增：

```text
frontend/src/api/inventory.ts
frontend/src/types/inventory.ts
frontend/src/pages/InventoryQuery.tsx
```

并更新：

```text
frontend/src/router/index.tsx
```

`/inventory` 页面要求：

- 搜索栏包含商品选择器、仓库下拉、库位下拉、查询按钮
- 库位选项随仓库筛选
- 表格只读
- 表格列为商品名称、SKU、仓库、库位、库存数量、更新时间
- 支持分页

前端不得新增库存流水菜单、路由、页面或表格。

---

## 5. Task Plan

### T6.1.1 实现 StockQueryService

修改范围：

- `backend/src/service/stock_query_service.rs`
- `backend/src/service/mod.rs`
- 如确有需要，可新增 `backend/src/dto/inventory.rs` 并注册

不得修改：

- `backend/src/router.rs`
- `backend/src/handler/*`
- `frontend/*`
- `backend/migrations/*`

验收依据：

- `TASK_BOARD.md` 的 T6.1.1
- 本文档的 Backend Design
- 能通过 `cargo fmt --check`
- 能通过 `cargo build`

### T6.1.2 实现 Inventory Handler 并注册路由

修改范围：

- `backend/src/handler/inventory_handler.rs`
- `backend/src/handler/mod.rs`
- `backend/src/router.rs`
- 必要的 DTO 补充

不得修改：

- `frontend/*`
- `backend/migrations/*`
- 入库、出库业务流程

验收依据：

- `TASK_BOARD.md` 的 T6.1.2
- 本文档的 Handler and Routes
- 能通过 `cargo fmt --check`
- 能通过 `cargo build`

### T6.2.1 实现库存查询页面

修改范围：

- `frontend/src/api/inventory.ts`
- `frontend/src/types/inventory.ts`
- `frontend/src/pages/InventoryQuery.tsx`
- `frontend/src/router/index.tsx`

不得修改：

- 后端业务逻辑
- 数据库迁移
- 新增库存流水前端页面

验收依据：

- `TASK_BOARD.md` 的 T6.2.1
- 本文档的 Frontend Design
- 能通过 `npm run build`

---

## 6. Review Checklist

Codex Review 必须检查：

- 是否只实现当前单一任务
- 是否符合 `TASK_BOARD.md` 当前任务验收标准
- 是否符合本 Epic6 实施边界
- 是否未修改 `openspec/specs/*`
- 是否未修改数据库 schema
- 是否未新增禁用端点或前端页面
- 是否未引入锁库存、审批流、分类能力
- 是否通过必要构建验证

---

## 7. Next Dispatch

当前唯一允许派发的下一任务：

```text
T6.1.1 实现 StockQueryService
```

派发时应引用：

```text
_ai/tasks/TASK_BOARD.md
_ai/tasks/Epic6-Implementation-Plan.md
```

不得复制大段验收标准，避免上下文膨胀。
